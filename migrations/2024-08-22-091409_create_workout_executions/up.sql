-- Your SQL goes here
CREATE TABLE WorkoutExecutions (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    workout_template_id UUID NOT NULL REFERENCES WorkoutTemplates(id) ON DELETE CASCADE,
    date DATE DEFAULT CURRENT_DATE NOT NULL,
    survey smallint NOT NULL CHECK (survey >= 0)
);
