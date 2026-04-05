.PHONY: run push build

run:
	npx nodemon --watch webpack.config.js --watch package.json --watch yarn.lock --ext js,json --exec "npx webpack --mode development --watch" & uv run flask --app app run --host 0.0.0.0 --port 8000 --debug

build:
	npx webpack --mode production

push:
	git remote | xargs -I R git push R master
