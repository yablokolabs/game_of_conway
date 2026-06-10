CREATE TABLE IF NOT EXISTS grid_requests (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    input_grid JSONB NOT NULL,
    output_grid JSONB NOT NULL,
    grid_size INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_grid_requests_user_id ON grid_requests (user_id);
CREATE INDEX idx_grid_requests_created_at ON grid_requests (created_at);
CREATE INDEX idx_grid_requests_grid_size ON grid_requests (grid_size);
