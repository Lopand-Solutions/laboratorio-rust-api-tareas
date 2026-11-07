-- Las tareas del laboratorio. Las reglas también viven en la base: si otro programa escribe aquí, siguen valiendo.
CREATE TABLE tareas (
    id        BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    titulo    TEXT NOT NULL CHECK (length(trim(titulo)) BETWEEN 1 AND 200),
    hecha     BOOLEAN NOT NULL DEFAULT false,
    prioridad SMALLINT NOT NULL DEFAULT 2 CHECK (prioridad BETWEEN 1 AND 3),
    creada    TIMESTAMPTZ NOT NULL DEFAULT now()
);
