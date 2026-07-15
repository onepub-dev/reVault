from setuptools import Distribution, setup


class BinaryDistribution(Distribution):
    """Tag wheels as platform-specific because they carry a native library."""

    def has_ext_modules(self) -> bool:
        return True


setup(distclass=BinaryDistribution)
