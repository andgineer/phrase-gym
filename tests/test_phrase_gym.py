from phrase_gym import __version__
from phrase_gym.main import phrase_gym
from click.testing import CliRunner


def test_version():
    assert __version__


def test_version_option():
    runner = CliRunner()
    result = runner.invoke(phrase_gym, ['--version'])
    assert result.exit_code == 0
    assert __version__ in result.output
