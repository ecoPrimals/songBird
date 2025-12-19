from setuptools import setup, find_packages

setup(
    name="ecoprimals-client",
    version="0.1.0",
    description="Student client for EcoPrimals federated ML",
    author="EcoPrimals Project",
    packages=find_packages(),
    install_requires=[
        "websockets>=12.0",
        "aiohttp>=3.9.0",
        "python-dateutil>=2.8.0",
        "rich>=13.0.0",
        "pydantic>=2.0.0",
    ],
    entry_points={
        "console_scripts": [
            "ecoprimals=ecoprimals_client.cli:main",
        ],
    },
    python_requires=">=3.8",
)

