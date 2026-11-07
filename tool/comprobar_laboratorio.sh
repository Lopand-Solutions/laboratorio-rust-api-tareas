#!/usr/bin/env bash
# Comprueba el laboratorio «API de tareas». Uso: ./tool/comprobar_laboratorio.sh
# Necesita un Postgres (DATABASE_URL en el entorno o en .env) para las pruebas de consultas.
# No toca tus archivos: los retos rompen el código en una copia temporal y la borran al terminar.
set -uo pipefail
cd "$(dirname "$0")/.."
RAIZ=$(pwd)
# La copia de los retos compila en el mismo target: así no recompila todas las dependencias.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$RAIZ/target}"
export RUST_BACKTRACE=0
if [ -z "${DATABASE_URL:-}" ] && [ -f .env ]; then
  DATABASE_URL=$(grep -E '^DATABASE_URL=' .env | head -1 | cut -d= -f2-)
  export DATABASE_URL
fi

bien=0
mal=0
check() { # check <código de salida> <texto si pasa> <texto si falla>
  if [ "$1" -eq 0 ]; then echo "  ✓ $2"; bien=$((bien + 1)); else echo "  ✗ $3"; mal=$((mal + 1)); fi
}

echo "1. Formato y clippy"
cargo fmt --check >/dev/null 2>&1
check $? "cargo fmt --check" "cargo fmt --check encontró diferencias: corre cargo fmt"
# Sin base: si clippy pasa así, la carpeta .sqlx tiene todas tus consultas.
salida=$(SQLX_OFFLINE=true cargo clippy --all-targets --quiet -- -D warnings 2>&1)
check $? "clippy sin avisos, compilando offline con .sqlx" \
  "clippy falló (si dice «no cached data», corre cargo sqlx prepare -- --all-targets):
$(echo "$salida" | grep -E '^(error|warning)' | head -5)"

echo "2. Pruebas sin base de datos"
salida=$(cargo test --lib --quiet 2>&1)
check $? "validación: $(echo "$salida" | grep -E '^test result' | head -1)" \
  "las pruebas de validación fallan:
$(echo "$salida" | grep -E 'panicked|FAILED|not yet implemented' | head -5)"

echo "3. Pruebas contra Postgres"
if [ -z "${DATABASE_URL:-}" ]; then
  check 1 "" "falta DATABASE_URL (en el entorno o en .env): sin Postgres no puedo correr tests/consultas.rs"
else
  salida=$(cargo test --test consultas --quiet 2>&1)
  check $? "consultas: $(echo "$salida" | grep -E '^test result' | head -1)" \
    "las pruebas de consultas fallan:
$(echo "$salida" | grep -E 'panicked|FAILED|not yet implemented' | head -6)"
fi

echo "4. Tus pruebas detectan los errores de los retos (en una copia)"
# archivo|línea original|línea rota|qué rompe
RETOS=(
  "src/tareas.rs|             titulo = COALESCE(\$2, titulo),|             titulo = \$2,|un PATCH que borra el título (reto 1)"
  "src/tareas.rs|        (pagina - 1) * POR_PAGINA|        pagina * POR_PAGINA|la paginación que se salta 20 tareas (reto 2)"
  "src/tareas.rs|    if (1..=3).contains(&prioridad) {|    if (0..=3).contains(&prioridad) {|una prioridad 0 que se cuela (reto 3)"
)
for reto in "${RETOS[@]}"; do
  IFS='|' read -r archivo original roto que <<<"$reto"
  if ! grep -qF -- "$original" "$archivo" 2>/dev/null; then
    check 1 "" "no encuentro en $archivo la línea «$(echo "$original" | xargs)»: ¿la cambiaste?"
    continue
  fi
  if [ -z "${DATABASE_URL:-}" ]; then
    check 1 "" "no puedo probar $que sin DATABASE_URL"
    continue
  fi
  copia=$(mktemp -d)
  tar --exclude=./target --exclude=./.git -cf - . | tar -xf - -C "$copia"
  contenido=$(cat "$copia/$archivo")
  printf '%s\n' "${contenido/"$original"/"$roto"}" >"$copia/$archivo"
  # Target aparte: si la copia rota compilara en el tuyo, dejaría su binario en target/debug.
  (cd "$copia" && CARGO_TARGET_DIR="$CARGO_TARGET_DIR/retos" cargo test --quiet >/dev/null 2>&1)
  r=$?
  rm -rf "$copia"
  [ "$r" -ne 0 ]
  check $? "detectan $que" "NO detectan $que: falta una prueba"
done

echo "5. Nada privado entra al repositorio"
grep -qxF ".env" .gitignore 2>/dev/null && grep -qxF "target/" .gitignore 2>/dev/null
check $? ".gitignore excluye .env y target/" ".gitignore debe tener las líneas .env y target/"
if [ -d .git ]; then
  ! git ls-files | grep -qE '(^|/)\.env$'
  check $? "git no tiene ningún .env" "git ya tiene un .env: sácalo con git rm --cached .env y cambia la contraseña"
fi

echo
if [ "$mal" -eq 0 ]; then
  echo "Laboratorio completo: $bien de $bien."
  exit 0
fi
[ "$mal" -eq 1 ] && echo "Falta 1 punto ($bien bien)." || echo "Faltan $mal puntos ($bien bien)."
exit 1
