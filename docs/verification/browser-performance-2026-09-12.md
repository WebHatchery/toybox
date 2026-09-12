# Browser and performance verification — 2026-09-12

The no-argument project publisher completed the Windows release build, WebGL
release build, asset packaging, and generated `dist/webgl/index.html`. The
generated page contains the visible touch-control copy and `rust_toybox` as the
Project Roost slug.

The required Preview deployment stopped before upload because the configured
WSL destination (`\\wsl.localhost\Ubuntu\home\kalai\dev\games\toybox`) is not
accessible from this managed Windows session. A project-local static server
also started successfully on port 8765, but the connected browser denied
permission to open the loopback URL. No browser or mobile screenshot claim is
made from that blocked attempt.

The native release benchmark still exercised the shipped 268-state shop for
10 seconds:

```text
BENCH toys=268 frames=1714 seconds=10.00 avg_fps=171.3 worst_frame_ms=80.97 slow_frames=1/1654 (>16.7ms, after 60 warm-up)
```

This is a native performance signal, not a substitute for the blocked mobile
WebGL boot/capture check. Re-run the browser step from a session that permits
the Preview host or loopback access, then capture title, gameplay, tool-shop,
pause, and recovery states at a mobile viewport.
