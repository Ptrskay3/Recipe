CREATE TYPE difficulty_level AS ENUM (
    'easy',
    'moderate',
    'medium',
    'challenging',
    'hard',
    'extreme',
    'do_not_attempt'
);

CREATE TABLE cuisines
(
    id UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    name TEXT NOT NULL COLLATE "case_insensitive" UNIQUE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ
);

SELECT trigger_updated_at('"cuisines"');

INSERT INTO cuisines (name) VALUES('Mexican');
INSERT INTO cuisines (name) VALUES('Swedish');
INSERT INTO cuisines (name) VALUES('Latvian');
INSERT INTO cuisines (name) VALUES('Italian');
INSERT INTO cuisines (name) VALUES('Spanish');
INSERT INTO cuisines (name) VALUES('American');
INSERT INTO cuisines (name) VALUES('Scottish');
INSERT INTO cuisines (name) VALUES('British');
INSERT INTO cuisines (name) VALUES('Thai');
INSERT INTO cuisines (name) VALUES('Japanese');
INSERT INTO cuisines (name) VALUES('Chinese');
INSERT INTO cuisines (name) VALUES('Indian');
INSERT INTO cuisines (name) VALUES('Canadian');
INSERT INTO cuisines (name) VALUES('Russian');
INSERT INTO cuisines (name) VALUES('Jewish');
INSERT INTO cuisines (name) VALUES('Polish');
INSERT INTO cuisines (name) VALUES('German');
INSERT INTO cuisines (name) VALUES('French');
INSERT INTO cuisines (name) VALUES('Hawaiian');
INSERT INTO cuisines (name) VALUES('Brazilian');
INSERT INTO cuisines (name) VALUES('Peruvian');
INSERT INTO cuisines (name) VALUES('Salvadorian');
INSERT INTO cuisines (name) VALUES('Cuban');
INSERT INTO cuisines (name) VALUES('Tibetan');
INSERT INTO cuisines (name) VALUES('Egyptian');
INSERT INTO cuisines (name) VALUES('Greek');
INSERT INTO cuisines (name) VALUES('Belgian');
INSERT INTO cuisines (name) VALUES('Irish');
INSERT INTO cuisines (name) VALUES('Welsh');
INSERT INTO cuisines (name) VALUES('Mormon');
INSERT INTO cuisines (name) VALUES('Cajun');
INSERT INTO cuisines (name) VALUES('Portuguese');
INSERT INTO cuisines (name) VALUES('Turkish');
INSERT INTO cuisines (name) VALUES('Haitian');
INSERT INTO cuisines (name) VALUES('Tahitian');
INSERT INTO cuisines (name) VALUES('Kenyan');
INSERT INTO cuisines (name) VALUES('Korean');
INSERT INTO cuisines (name) VALUES('Algerian');
INSERT INTO cuisines (name) VALUES('Nigerian');
INSERT INTO cuisines (name) VALUES('Libyan');
INSERT INTO cuisines (name) VALUES('Hungarian');
INSERT INTO cuisines (name) VALUES('Unspecified');

CREATE TYPE type_by_time AS ENUM (
    'breakfast',
    'lunch',
    'dinner',
    'other'
);

CREATE TABLE recipes
(
    id UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    name TEXT NOT NULL COLLATE "case_insensitive" UNIQUE,
    description TEXT NOT NULL,
    creator_id UUID NOT NULL REFERENCES "users" (user_id) ON DELETE CASCADE,
    prep_time INT NOT NULL,
    cook_time INT NOT NULL,
    difficulty difficulty_level NOT NULL,
    steps TEXT[] NOT NULL,
    cuisine_id UUID NOT NULL REFERENCES "cuisines" (id) ON DELETE CASCADE,
    meal_type type_by_time NOT NULL,
    created_at    TIMESTAMPTZ                            NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ
);

SELECT trigger_updated_at('"recipes"');
