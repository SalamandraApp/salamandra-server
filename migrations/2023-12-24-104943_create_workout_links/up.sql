-- Your SQL goes here
CREATE TABLE workout_linkers (
    id SERIAL PRIMARY KEY,
    workout_id BIGINT UNSIGNED NOT NULL,
    exercise_id BIGINT UNSIGNED NOT NULL,
    FOREIGN KEY (workout_id) REFERENCES workouts(id),
    FOREIGN KEY (exercise_id) REFERENCES exercises(id)
);
