import asyncio
import math
import json
from datetime import datetime, timezone


def make_image_request(
    request_id: int,
    T_world_camera: list[float],
    camera_id: bool = False,
    intrinsics: bool = False,
    timestamp: str | None = None,
) -> dict:
    assert len(T_world_camera) == 7, "T_world_camera must be [x, y, z, qw, qx, qy, qz]"
    return {
        "request_id": request_id,
        "timestamp": timestamp or datetime.now(timezone.utc).isoformat(),
        "camera_id": camera_id,
        "T_world_camera": T_world_camera,
        "intrinsics": intrinsics,
    }


def look_at_origin_quat(pos: tuple[float, float, float]) -> tuple[float, float, float, float]:
    """Quaternion [qw, qx, qy, qz] orienting a +X-forward, +Z-up camera at `pos`
    so that +X points toward the origin."""
    x, y, z = pos
    n = math.sqrt(x * x + y * y + z * z)
    if n == 0.0:
        return (1.0, 0.0, 0.0, 0.0)
    fx, fy, fz = -x / n, -y / n, -z / n  # forward = toward origin

    world_up = (0.0, 0.0, 1.0)
    # If forward is parallel to world_up, pick another up reference.
    if abs(fz) > 0.999:
        world_up = (0.0, 1.0, 0.0)

    # left = world_up x forward  (Y axis in right-handed X-fwd, Z-up frame)
    lx = world_up[1] * fz - world_up[2] * fy
    ly = world_up[2] * fx - world_up[0] * fz
    lz = world_up[0] * fy - world_up[1] * fx
    ln = math.sqrt(lx * lx + ly * ly + lz * lz)
    lx, ly, lz = lx / ln, ly / ln, lz / ln

    # up = forward x left  (Z axis)
    ux = fy * lz - fz * ly
    uy = fz * lx - fx * lz
    uz = fx * ly - fy * lx

    # Rotation matrix columns: X=forward, Y=left, Z=up
    m00, m01, m02 = fx, lx, ux
    m10, m11, m12 = fy, ly, uy
    m20, m21, m22 = fz, lz, uz

    trace = m00 + m11 + m22
    if trace > 0.0:
        s = math.sqrt(trace + 1.0) * 2.0
        qw = 0.25 * s
        qx = (m21 - m12) / s
        qy = (m02 - m20) / s
        qz = (m10 - m01) / s
    elif m00 > m11 and m00 > m22:
        s = math.sqrt(1.0 + m00 - m11 - m22) * 2.0
        qw = (m21 - m12) / s
        qx = 0.25 * s
        qy = (m01 + m10) / s
        qz = (m02 + m20) / s
    elif m11 > m22:
        s = math.sqrt(1.0 + m11 - m00 - m22) * 2.0
        qw = (m02 - m20) / s
        qx = (m01 + m10) / s
        qy = 0.25 * s
        qz = (m12 + m21) / s
    else:
        s = math.sqrt(1.0 + m22 - m00 - m11) * 2.0
        qw = (m10 - m01) / s
        qx = (m02 + m20) / s
        qy = (m12 + m21) / s
        qz = 0.25 * s
    return (qw, qx, qy, qz)


def fibonacci_sphere(n: int, radius: float) -> list[tuple[float, float, float]]:
    """Roughly even point distribution on a sphere of `radius`."""
    points = []
    phi = math.pi * (3.0 - math.sqrt(5.0))  # golden angle
    for i in range(n):
        y = 1.0 - (i / max(n - 1, 1)) * 2.0
        r = math.sqrt(max(0.0, 1.0 - y * y))
        theta = phi * i
        points.append((radius * math.cos(theta) * r, radius * y, radius * math.sin(theta) * r))
    return points


async def send_one(request_id: int, pos: tuple[float, float, float], sock_path: str) -> None:
    qw, qx, qy, qz = look_at_origin_quat(pos)
    req = make_image_request(
        request_id=request_id,
        T_world_camera=[pos[0], pos[1], pos[2], qw, qx, qy, qz],
    )
    _reader, writer = await asyncio.open_unix_connection(sock_path)
    try:
        writer.write((json.dumps(req) + '\n').encode())
        await writer.drain()
    finally:
        writer.close()
        try:
            await writer.wait_closed()
        except Exception:
            pass


async def send_sphere(radius: float, n_points: int, sock_path: str = '/tmp/splatcom.sock') -> None:
    tasks = [
        asyncio.create_task(send_one(i, pos, sock_path))
        for i, pos in enumerate(fibonacci_sphere(n_points, radius))
    ]
    await asyncio.gather(*tasks)


if __name__ == '__main__':
    asyncio.run(send_sphere(radius=5.0, n_points=64))
