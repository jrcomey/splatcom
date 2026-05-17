import asyncio
import math
import json
from datetime import datetime, timezone


# brush is COLMAP convention:
#   camera-local +X = right, +Y = down, +Z = forward
#   T_world_camera rotation = camera-to-world (cam-local axes in world)
_SPHERE_CENTER = (-6.285185-3, -5.5856934-3, -14.807054-2)
_LOOK_AT_CENTER = (-6.285185, -5.5856934, -14.807054)
_WORLD_UP = (0.0, 0.0, 1.0) # This is not COLMAP but the test image is +Z 


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


def look_at_quat(
    eye: tuple[float, float, float],
    target: tuple[float, float, float],
    world_up: tuple[float, float, float] = _WORLD_UP,
) -> tuple[float, float, float, float]:
    """Cam-to-world quaternion [qw, qx, qy, qz] aiming a COLMAP camera
    (+X right, +Y down, +Z forward) at `target` from `eye`."""
    fx, fy, fz = target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]
    fn = math.sqrt(fx * fx + fy * fy + fz * fz)
    if fn == 0.0:
        return (1.0, 0.0, 0.0, 0.0)
    fx, fy, fz = fx / fn, fy / fn, fz / fn

    ux, uy, uz = world_up
    # forward colinear with world_up — pick any perpendicular fallback.
    if abs(fx * ux + fy * uy + fz * uz) > 0.999:
        ux, uy, uz = (0.0, 0.0, 1.0) if abs(uz) < 0.5 else (1.0, 0.0, 0.0)

    # right = normalize(forward x world_up); RHR with +X right, +Y down, +Z fwd.
    rx = fy * uz - fz * uy
    ry = fz * ux - fx * uz
    rz = fx * uy - fy * ux
    rn = math.sqrt(rx * rx + ry * ry + rz * rz)
    rx, ry, rz = rx / rn, ry / rn, rz / rn

    # down = forward x right
    dx = fy * rz - fz * ry
    dy = fz * rx - fx * rz
    dz = fx * ry - fy * rx

    # R_c2w columns: right (cam +X), down (cam +Y), forward (cam +Z)
    m00, m01, m02 = rx, dx, fx
    m10, m11, m12 = ry, dy, fy
    m20, m21, m22 = rz, dz, fz

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


def fibonacci_sphere(
    n: int,
    radius: float,
    center: tuple[float, float, float] = _SPHERE_CENTER,
) -> list[tuple[float, float, float]]:
    """Roughly even point distribution on a sphere of `radius` around `center`."""
    points = []
    phi = math.pi * (3.0 - math.sqrt(5.0))  # golden angle
    for i in range(n):
        y = 1.0 - (i / max(n - 1, 1)) * 2.0
        r = math.sqrt(max(0.0, 1.0 - y * y))
        theta = phi * i
        points.append((
            radius * math.cos(theta) * r + center[0],
            radius * y + center[1],
            radius * math.sin(theta) * r + center[2],
        ))
    return points


async def send_one(
    request_id: int,
    pos: tuple[float, float, float],
    target: tuple[float, float, float],
    host: str,
    port: int,
) -> None:
    qw, qx, qy, qz = look_at_quat(pos, target)
    req = make_image_request(
        request_id=request_id,
        T_world_camera=[pos[0], pos[1], pos[2], qw, qx, qy, qz],
    )
    reader, writer = await asyncio.open_connection(host, port)
    try:
        writer.write((json.dumps(req) + '\n').encode())
        await writer.drain()
        line = await reader.readline()
        if line:
            resp = json.loads(line.decode())
            print(f"Response for request_id={request_id}")
            for k, v in resp.items():
                print(f"{k}: {v}")
    finally:
        writer.close()
        try:
            await writer.wait_closed()
        except Exception:
            pass


async def send_sphere(
    radius: float,
    n_points: int,
    sphere_center: tuple[float, float, float] = _SPHERE_CENTER,
    look_at_center: tuple[float, float, float] = _LOOK_AT_CENTER,
    host: str = '127.0.0.1',
    port: int = 8080,
) -> None:
    tasks = [
        asyncio.create_task(send_one(i, pos, look_at_center, host, port))
        for i, pos in enumerate(fibonacci_sphere(n_points, radius, sphere_center))
    ]
    await asyncio.gather(*tasks)


if __name__ == '__main__':
    asyncio.run(send_sphere(radius=10.0, n_points=64))
