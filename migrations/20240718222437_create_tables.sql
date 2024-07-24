-- Create the rooms table
CREATE TABLE rooms (
    id SERIAL PRIMARY KEY,
    game_stage VARCHAR(50) NOT NULL DEFAULT 'waiting_for_players',
    current_team VARCHAR(50) NOT NULL DEFAULT 'red',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create the players table
CREATE TABLE players (
    id SERIAL PRIMARY KEY,
    room_id INTEGER NOT NULL,
    username VARCHAR(255) NOT NULL,
    team VARCHAR(50) NOT NULL DEFAULT 'neutral',
    role VARCHAR(50) NOT NULL DEFAULT 'guesser',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_room
    FOREIGN KEY (room_id)
    REFERENCES rooms (id)
    ON DELETE CASCADE
);

-- Create the field table
CREATE TABLE fields (
    id SERIAL PRIMARY KEY,
    room_id INTEGER NOT NULL,
    team VARCHAR(50) NOT NULL,
    text VARCHAR(255) NOT NULL,
    is_used BOOLEAN DEFAULT FALSE NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_room
    FOREIGN KEY (room_id)
    REFERENCES rooms (id)
    ON DELETE CASCADE
);
