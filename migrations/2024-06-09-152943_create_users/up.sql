-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE TABLE Users (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    username VARCHAR UNIQUE NOT NULL,
    display_name VARCHAR NOT NULL,
    date_joined DATE DEFAULT CURRENT_DATE NOT NULL,
    date_of_birth DATE,
    height smallint CHECK (height >= 0),
    weight FLOAT4 CHECK (weight >= 0),
    gender smallint CHECK (gender >= 0),
    fitness_goal smallint CHECK (fitness_goal >= 0),
    fitness_level smallint CHECK (fitness_level >= 0)
);
