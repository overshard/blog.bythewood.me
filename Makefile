.PHONY: run push build

run:
	bun run dev & uv run flask --app app run --host 0.0.0.0 --port 8000 --debug

build:
	bun run build

push:
	git remote | xargs -I R git push R master
