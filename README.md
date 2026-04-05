# Blog

A self-hostable blog built on Wagtail for developers.


## Motivation

I was bored and felt like writing my own blog over the weekend.


## Features

- Top notch SEO using industry best practices and multiple scanners to detect
  issues on a regular basis.
- Built on Wagtail and Django, powering some of the largest websites in the
  world. Wagtail is used by Google, NASA, MIT, Mozilla, and many others.
- Customized for developers, makes use of CodeMirror for syntax highlighting and
  on the admin panel for a clean code editing experience.
- Easily adjusted to fit your needs with the use of well documented projects
  like the above mentioned Wagtail and Django plus Bootstrap for the design and
  easy to adjust hosting options using Docker.
- Minimal network payloads, out of the box 100% scores on all lighthouse metrics
  and most pages, even with a few images, are less than 300kB in size.


## Requirements

You need docker + docker-compose installed for a quick production start or you
can figure out how we install and run things via the `Dockerfile` and set it up
yourself.

If you want to install things without docker then you'll need the following
dependencies:

- python
- pipenv
- node
- yarn
- chromium

You can also check the `Dockerfile` for an exact list of dependencies and adjust
package names for your desired platform.

This is a standard Django project. If you know how to run Django, or want to
look up any Django tutorial on how to run Django, you shouldn't have a problem
getting this project running on almost anything.


## Running locally

If you have all of the above dependencies installed you can use my Makefile to
run and install python and node dependencies locally. Running `make` will check
that you have the proper dependencies installed and if not it will try and
install them for you. It will then create you a fresh database and run
everything.


## Checking outdated dependencies

This can be done in both yarn and pipenv with the following two commands:

    pipenv update --outdated
    yarn outdated

You can then upgrade the outdated dependencies with the following two commands:

    pipenv update
    yarn upgrade

I recommend testing everything after this to make sure it's all working.


## Optimizing images with webp

My development system runs Ubuntu so I installed the official webp utils from
Google with `apt install webp`.

    cwebp -q 90 -m 6 -o output.webp input.png


## Using docker-compose

The easiest way to run this project is to run it using
`docker-compose up --build -d` if you have `docker-compose` and `docker`
installed. This will start the server and have you running at port 8000. The
first time you do this make sure you run migrations with
`docker-compose run web python manage.py migrate`. Make sure you setup the
`.env` file before running, you can copy the sample from
`samplefiles/env.sample` into the root of the project as `.env` and change the
variables.


## Default user

The default user is `admin` with the password `admin`.


## Backups

All data is stored in `/srv/data/blog.bythewood.me/` and your repo is stored in
`/srv/git/blog.bythewood.me.git/`. You can backup both of these folders and you'll have
a 100% backup of everything except changes you may have made to the `Caddyfile`
and the `.env` file which should be easy enough to recreate but you can back
those up too!


## Support

I won't be providing any user support for this project. I'm more than happy to
accept good pull requests and fix bugs but I don't have the time to help people
run or use this project. I appologize in advance for this. Maintaining
mutliple OSS projects has taught me that I need to step back from trying to
provide support to avoid burnout.


## Server guide

This quickstart requires that you have an Alpine Linux server running with a
domain name pointed to it. I'm currently using Linode as my host since they
support Alpine Linux nicely. If you don't want to use Linode or Alpine Linux
you can use these instructions and just change the apk commands at the start to
whatever Linux distro you're using.

**IMPORTANT NOTE**: Change `blog.bythewood.me` to your domain name where
relevant in these instructions.

**TIP**: During the ufw portion to enable the firewall I recommend only allowing
your IP address or your ISP's IP address range which you can find on whois
lookups at the top. For example, replace `192.230.176.0/20` with your IP or your
ISP's IP range.

    ufw allow from 192.230.176.0/20 proto tcp to any port 22

I allow my local ISP's range because I have a DHCP lease from them and I get
tired of logging into my server from my hosting provider's UI to update it. It's
good enough security and much better than nothing!

Server:

    apk update && apk upgrade && apk add docker docker-compose caddy git iptables ip6tables ufw
    ufw allow 22/tcp && ufw allow 80/tcp && ufw allow 443/tcp && ufw --force enable
    echo -e "#!/bin/sh\napk upgrade --update | sed \"s/^/[\`date\`] /\" >> /var/log/apk-autoupgrade.log" > /etc/periodic/daily/apk-autoupgrade && chmod 700 /etc/periodic/daily/apk-autoupgrade
    rc-update add docker boot && service docker start
    mkdir -p /srv/git/blog.bythewood.me.git && cd /srv/git/blog.bythewood.me.git && git init --bare

Local:

    git clone git@github.com:overshard/blog.bythewood.me.git && cd blog.bythewood.me
    git remote remove origin && git remote add origin root@blog.bythewood.me:/srv/git/blog.bythewood.me.git
    git push --set-upstream origin master

Server:

    mkdir -p /srv/docker && cd /srv/docker && git clone /srv/git/blog.bythewood.me.git blog.bythewood.me && cd /srv/docker/blog.bythewood.me
    cp samplefiles/Caddyfile.sample /etc/caddy/Caddyfile && sed -i 's/blog.example.com/blog.bythewood.me/g' /etc/caddy/Caddyfile
    cp samplefiles/env.sample .env && sed -i 's/blog.example.com/blog.bythewood.me/g' .env
    cp samplefiles/post-receive.sample /srv/git/blog.bythewood.me.git/hooks/post-receive
    mkdir -p /srv/data/blog.bythewood.me/db && chown -R 1000:1000 /srv/data/blog.bythewood.me
    docker-compose up --build --detach && docker-compose run web python3 manage.py migrate --noinput && docker-compose run web sqlite3 db.sqlite3 "PRAGMA journal_mode=WAL;" ".exit"
    rc-update add caddy boot && service caddy start
