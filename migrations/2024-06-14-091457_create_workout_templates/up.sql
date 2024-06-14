-- Your SQL goes here
CREATE TABLE WorkoutTemplates (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES Users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    date_created DATE NOT NULL
);
