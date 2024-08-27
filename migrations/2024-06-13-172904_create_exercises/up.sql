-- Your SQL goes here
CREATE TABLE Exercises (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    main_muscle_group smallint CHECK (main_muscle_group >= 0),
    secondary_muscle_group smallint CHECK (secondary_muscle_group >= 0),
    necessary_equipment smallint CHECK (necessary_equipment >= 0),
    exercise_type smallint CHECK (exercise_type >= 0)
);
