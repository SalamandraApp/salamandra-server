-- Your SQL goes here
CREATE TABLE WKExecutionElements (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    workout_execution_id UUID NOT NULL REFERENCES WorkoutEXecutions(id) ON DELETE CASCADE,
    exercise_id UUID NOT NULL REFERENCES Exercises(id) ON DELETE CASCADE,
    position smallint NOT NULL CHECK (position >= 0),
    exercise_number smallint NOT NULL CHECK (exercise_number >= 0),
    reps smallint NOT NULL CHECK (reps >= 0),
    set_number smallint NOT NULL CHECK (set_number >= 0),
    weight FLOAT4 CHECK (weight >= 0),
    rest smallint NOT NULL CHECK (rest >= 0),
    super_set smallint CHECK (super_set >= 0),
    time INT NOT NULL CHECK (time >= 0)
);
