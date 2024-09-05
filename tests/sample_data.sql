-- Clean the database
TRUNCATE TABLE WKTemplateElements, WorkoutTemplates, Exercises, Users RESTART IDENTITY CASCADE;

-- Insert users
INSERT INTO Users (id, username, display_name)
VALUES
('f139d0fe-10a1-700d-1247-836eed5e053b', 'pato', 'Pato'),
('81c9608e-30c1-701c-e7db-f81caa667ea2', 'vzkz', 'vzkz');

-- Insert exercises
INSERT INTO Exercises (id, name, main_muscle_group, secondary_muscle_group, necessary_equipment, exercise_type)
VALUES
(gen_random_uuid(), 'Push Up', 2, 3, 2, 1),
(gen_random_uuid(), 'Pull Up', 4, 2, 1, 1),
(gen_random_uuid(), 'Squat', 4, 2, 1, 2);

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
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout A'), (SELECT id FROM Exercises WHERE name = 'Push Up'),  1, 6, 4, 50, 50, NULL),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout A'), (SELECT id FROM Exercises WHERE name = 'Pull Up'),  2, 6, 3, 50, 60, NULL),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout A'), (SELECT id FROM Exercises WHERE name = 'Squat'),    3, 6, 7, 50, 60, NULL),

(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout B'), (SELECT id FROM Exercises WHERE name = 'Push Up'),  1, 6, 4, 50, 60, NULL),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout B'), (SELECT id FROM Exercises WHERE name = 'Pull Up'),  2, 6, 4, 50, 60, NULL),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout B'), (SELECT id FROM Exercises WHERE name = 'Squat'),    3, 6, 4, 50, 60, NULL),

(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout C'), (SELECT id FROM Exercises WHERE name = 'Push Up'),  1, 6, 4, 30, 60, NULL),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout C'), (SELECT id FROM Exercises WHERE name = 'Pull Up'),  2, 6, 4, 50, 100, NULL),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout C'), (SELECT id FROM Exercises WHERE name = 'Squat'),    3, 12, 4, 50, 60, NULL),

(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout D'), (SELECT id FROM Exercises WHERE name = 'Push Up'),  1, 6, 4, 50, 60, NULL),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout D'), (SELECT id FROM Exercises WHERE name = 'Pull Up'),  2, 12, 4, 20, 60, NULL),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout D'), (SELECT id FROM Exercises WHERE name = 'Squat'),    3, 4, 4, 50, 90, NULL);

-- Insert workout templates
INSERT INTO WorkoutExecutions(id, workout_template_id, date, survey)
VALUES
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout A'), CURRENT_DATE, 0),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout B'), CURRENT_DATE, 0),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout C'), CURRENT_DATE, 0),
(gen_random_uuid(), (SELECT id FROM WorkoutTemplates WHERE name = 'Workout D'), CURRENT_DATE, 0);
