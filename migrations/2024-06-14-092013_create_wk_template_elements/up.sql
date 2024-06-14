-- Your SQL goes here
CREATE TABLE WKTemplateElements (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    workout_template_id UUID NOT NULL REFERENCES WorkoutTemplates(id) ON DELETE CASCADE,
    exercise_id UUID NOT NULL REFERENCES Exercises(id) ON DELETE CASCADE,
    position INT NOT NULL,
    reps INT NOT NULL,
    sets INT NOT NULL,
    weight INT NOT NULL,
    rest INT NOT NULL,
    super_set INT
);
