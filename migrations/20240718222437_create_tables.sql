-- Create the rooms table
CREATE TABLE rooms (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create the players table
CREATE TABLE players (
    id SERIAL PRIMARY KEY,
    room_id INTEGER NOT NULL,
    username VARCHAR(255) NOT NULL,
    team VARCHAR(50),
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
    team VARCHAR(50),
    text VARCHAR(255),
    is_used BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_room
    FOREIGN KEY (room_id)
    REFERENCES rooms (id)
    ON DELETE CASCADE
);
