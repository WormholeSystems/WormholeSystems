#!/bin/sh
# Database dumps. The `backup` service runs `schedule`; `wsctl backup`, `wsctl restore` and
# `wsctl update` run the other commands in a one-off container of the same service, so the
# same client and the same connection do both. DATABASE_URL is whichever database the API
# uses, and /backups is WS_BACKUP_DIR on the host.
set -eu
umask 077
dir=/backups

usage() {
	echo "usage: ws-backup schedule | dump [label] | restore FILE | server-version" >&2
	exit 2
}

dump() {
	name="wormholesystems-$(date -u +%Y%m%d-%H%M%S)-$1.dump"
	# Under a hidden name until it is complete: a dump cut short must never pass for one.
	pg_dump --format=custom --no-owner --no-acl --file="$dir/.$name.partial" "$DATABASE_URL"
	mv "$dir/.$name.partial" "$dir/$name"
	# This runs as root; whoever owns the directory should be able to copy the dump off.
	chown "$(stat -c %u:%g "$dir")" "$dir/$name" 2>/dev/null || true
	echo "$name"
}

prune() {
	find "$dir" -maxdepth 1 -name 'wormholesystems-*.dump' \
		-mtime "+${WS_BACKUP_KEEP_DAYS:-14}" -print -delete
}

case "${1:-schedule}" in
schedule)
	hours="${WS_BACKUP_EVERY_HOURS:-24}"
	echo "a dump every ${hours}h into $dir, kept for ${WS_BACKUP_KEEP_DAYS:-14} days"
	while :; do
		dump scheduled || echo "the dump failed; trying again in ${hours}h" >&2
		prune
		sleep $((hours * 3600))
	done
	;;
dump)
	dump "${2:-manual}"
	;;
restore)
	[ $# -eq 2 ] || usage
	[ -f "$2" ] || {
		echo "no dump at $2" >&2
		exit 1
	}
	# Everything the API makes lives in public. Emptied first, so the dump does not meet
	# tables from a newer migration it knows nothing about.
	psql -X --quiet -v ON_ERROR_STOP=1 -c 'set client_min_messages = warning' \
		-c 'drop schema public cascade' -c 'create schema public' "$DATABASE_URL"
	pg_restore --no-owner --no-acl --exit-on-error --dbname="$DATABASE_URL" "$2"
	;;
server-version)
	psql -XAtc 'show server_version_num' "$DATABASE_URL"
	;;
*)
	usage
	;;
esac
