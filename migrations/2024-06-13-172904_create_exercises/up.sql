-- Your SQL goes here
CREATE TABLE Exercises (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    main_muscle_group INT,
    secondary_muscle_group INT,
    necessary_equipment INT,
    exercise_type INT
);
