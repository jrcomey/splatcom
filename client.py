from sender import *
import asyncio

if __name__ == "__main__":
    asyncio.run(send_sphere(radius=10.0, n_points=64))