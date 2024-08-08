-- public.douban_game definition

-- Drop table

-- DROP TABLE public.douban_game;

CREATE TABLE public.douban_game (
	id bigserial NOT NULL,
	created_at timestamp DEFAULT CURRENT_TIMESTAMP NOT NULL,
	games jsonb NULL,
	CONSTRAINT douban_game_pk PRIMARY KEY (id)
);