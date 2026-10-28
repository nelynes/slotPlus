#!/bin/sh
SD="${SLOT_ROOT:-/mnt/sdcard}"
SYS="$SD/System"
RUN="${AGS_RUN:-/run}"
LOGS="$SD/Logs"
SESSION_MAX=65536
export SLOT_ROOT="$SD"
export SLOT_RUN="$RUN"

mkdir -p "$LOGS" 2>/dev/null
for f in slot.log slot.log.1 slot-session.log slot-session.log.1 slot-card.log ags-shutdown-trace.log input-trace.log; do
	[ -f "$SD/$f" ] && mv -f "$SD/$f" "$LOGS/$f"
done
[ -d "$SD/crash" ] && [ ! -e "$LOGS/crash" ] && mv "$SD/crash" "$LOGS/crash"

[ $(wc -c < "$LOGS/slot-session.log" 2>/dev/null || echo 0) -gt "$SESSION_MAX" ] \
	&& mv -f "$LOGS/slot-session.log" "$LOGS/slot-session.log.1"

log() {
	echo "$(date '+%F %T') $*" >> "$LOGS/slot-session.log" 2>/dev/null
}

pid="$(cat "$RUN/slot-led.pid" 2>/dev/null)"
if [ -z "$pid" ] || [ ! -d "/proc/$pid" ]; then
	sh "$SYS/led.sh" once
	sh "$SYS/led.sh" loop > /dev/null 2>&1 &
	echo $! > "$RUN/slot-led.pid"
fi

pid="$(cat "$RUN/slot-lid.pid" 2>/dev/null)"
if [ -z "$pid" ] || [ ! -d "/proc/$pid" ]; then
	sh "$SYS/lid.sh" loop > /dev/null 2>&1 &
	echo $! > "$RUN/slot-lid.pid"
fi

echo "|/bin/sh $SYS/coresave.sh %p %s %e %t" > /proc/sys/kernel/core_pattern 2>/dev/null

for gov in /sys/devices/system/cpu/cpufreq/policy*/scaling_governor; do
	echo schedutil > "$gov" 2>/dev/null || log "governor: schedutil refused by $gov"
done

[ -f "$RUN/slotlink.session" ] && sh "$SYS/slotlink.sh" link down > /dev/null 2>&1

# Nintendo DS speed test. DS games put in Bench/ are timed once with DSperate,
# before slot starts, into Bench/results.txt: each runs from boot with no input
# (a long warm-up gets past the intros), then the frames are timed, then its
# save state is weighed. Delete results.txt to time them again. A game that
# hangs is given up on after ten minutes.
BENCH="$SD/Bench"
if ls "$BENCH"/*.nds > /dev/null 2>&1 && [ ! -f "$BENCH/results.txt" ] \
	&& [ -f "$SYS/core_bench" ] && [ -f "$SYS/dsperate_libretro.so" ]; then
	log "bench $BENCH"
	limit=""
	command -v timeout > /dev/null 2>&1 && limit="timeout 600"
	bench() {
		game="$1"
		shift
		$limit /lib/ld-linux-aarch64.so.1 "$SYS/core_bench" "$SYS/dsperate_libretro.so" \
			"$game" --system "$SD/BIOS" "$@"
	}
	{
		echo "DSperate speed test, $(date '+%F %T')"
		grep -m1 -i "^hardware" /proc/cpuinfo
		echo "Needs 16.7 ms/frame or less (60 frames/s) for full speed."
		for rom in "$BENCH"/*.nds; do
			echo
			echo "== $(basename "$rom")"
			bench "$rom" --warmup 1200 --frames 600 --repeats 2 2>&1 | sed '1d'
			bench "$rom" --warmup 1200 --frames 120 --serialize 2>&1 | head -1
		done
	} > "$BENCH/results.txt.part" 2>&1
	mv -f "$BENCH/results.txt.part" "$BENCH/results.txt"
	log "bench done"
fi

[ -f "$LOGS/slot.log" ] && mv -f "$LOGS/slot.log" "$LOGS/slot.log.1"

# An empty Config/trace on the card turns on slot's timing log: every five
# seconds of play, the core's frame rate and frame times, and the sound
# buffer's level, into Logs/slot.log.
[ -f "$SD/Config/trace" ] && export SLOT_TRACE=1 && log "trace on"
log "exec $SYS/slot"

exec /lib/ld-linux-aarch64.so.1 "$SYS/slot" > "$LOGS/slot.log" 2>&1
