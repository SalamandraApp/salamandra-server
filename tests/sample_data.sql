-- Clean the database
TRUNCATE TABLE WKTemplateElements, WorkoutTemplates, Exercises, Users RESTART IDENTITY CASCADE;

-- Insert users
INSERT INTO Users (id, username, display_name)
VALUES
('f139d0fe-10a1-700d-1247-836eed5e053b', 'pato', 'Pato'),
('81c9608e-30c1-701c-e7db-f81caa667ea2', 'vzkz', 'vzkz'););

-- Insert exercises
INSERT INTO Exercises (id, name, main_muscle_group, secondary_muscle_group, necessary_equipment, exercise_type)
VALUES
(gen_random_uuid(), 'Push Up', 1, 1, 1, 1),
(gen_random_uuid(), 'Pull Up', 1, 1, 1, 1),
(gen_random_uuid(), 'Squat', 1, 1, 1, 1);

-- Insert workout templates
INSERT INTO WorkoutTemplates (id, user_id, name, description, date_created)
VALUES
(gen_random_uuid(), (SELECT id FROM Users WHERE username = 'pato'), 'Workout A', '', CURRENT_DATE),
(gen_random_uuid(), (SELECT id FROM Users WHERE username = 'pato'), 'Workout B', '', CURRENT_DATE),
(gen_random_uuid(), (SELECT id FROM Users WHERE username = 'vzkz'), 'Workout C', '', CURRENT_DATE),
(gen_random_uuid(), (SELECT id FROM Users WHERE username = 'vzkz'), 'Workout D', '', CURRENT_DATE);

-- Insert workout template elements
INSERT INTO WKTemplateElements (id, workout_template_id, exercise_id, position, reps, sets, weight, rest, super_set)
VALUES
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout A'), (SELECT id FROM Exercises WHERE name = 'Push Up'),  1, 1, 1, 1, 1, NULL),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout A'), (SELECT id FROM Exercises WHERE name = 'Pull Up'),  1, 1, 1, 1, 1, NULL),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout A'), (SELECT id FROM Exercises WHERE name = 'Squat'),    1, 1, 1, 1, 1, NULL),

(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout B'), (SELECT id FROM Exercises WHERE name = 'Push Up'),  1, 1, 1, 1, 1, NULL),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout B'), (SELECT id FROM Exercises WHERE name = 'Pull Up'),  1, 1, 1, 1, 1, NULL),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout B'), (SELECT id FROM Exercises WHERE name = 'Squat'),    1, 1, 1, 1, 1, NULL),

(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout C'), (SELECT id FROM Exercises WHERE name = 'Push Up'),  1, 1, 1, 1, 1, NULL),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout C'), (SELECT id FROM Exercises WHERE name = 'Pull Up'),  1, 1, 1, 1, 1, NULL),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout C'), (SELECT id FROM Exercises WHERE name = 'Squat'),    1, 1, 1, 1, 1, NULL),

(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout D'), (SELECT id FROM Exercises WHERE name = 'Push Up'),  1, 1, 1, 1, 1, NULL),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout D'), (SELECT id FROM Exercises WHERE name = 'Pull Up'),  1, 1, 1, 1, 1, NULL),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout D'), (SELECT id FROM Exercises WHERE name = 'Squat'),    1, 1, 1, 1, 1, NULL);

-- Insert workout templates
INSERT INTO WorkoutExecutions(id, workout_template_id, date, survey)
VALUES
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout A'), CURRENT_DATE, 0),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout B'), CURRENT_DATE, 0),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout C'), CURRENT_DATE, 0),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout D'), CURRENT_DATE, 0);
