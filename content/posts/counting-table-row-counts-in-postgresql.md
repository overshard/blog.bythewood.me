---
title: Counting table row counts in PostgreSQL
slug: counting-table-row-counts-in-postgresql
date: 2022-05-28
publish_date: 2022-05-28
tags: databases
description: An easy way to count the number of rows in a PostgreSQL table and sort by totals allowing you to find what's taking up space in your database.
cover_image: postgresql-row-count-output.webp
---

I sometimes find myself running into the problem of hunting down what is taking up a lot of rows in PostgreSQL due to service row restrictions. There is a choice of increasing my service plan but I sometimes find it unnecessary if I have a rogue app just adding a lot of data that can be purged. This happens a lot of logs and security apps tracking login attempts. To find the number of rows used in a PostgreSQL database and order it by count you can run this in `psql`.

```shell
SELECT nspname AS schemaname,relname,reltuples
FROM pg_class C
LEFT JOIN pg_namespace N ON (N.oid = C.relnamespace)
WHERE nspname NOT IN ('pg_catalog', 'information_schema')
AND relkind='r'
ORDER BY reltuples DESC;
```

If this runs correctly you should see a sorted list of tables with their row counts. From there you can create a script to purge the offending apps on a schedule if you don't need older data.
