import json
from pathlib import Path

from mwutil.local_config import MWUtilConfig


def get_data_file(config: MWUtilConfig) -> Path:
    return config.basedir / ".mwutil.data.json"

def get_data_entry(config: MWUtilConfig, key: str, default=None):
    data_file = get_data_file(config)
    if not data_file.exists():
        return default

    try:
        with data_file.open("r", encoding="utf-8") as f:
            data = json.load(f)
        return data.get(key, default)
    except (json.JSONDecodeError, OSError):
        return default

def set_data_entry(config: MWUtilConfig, key: str, value):
    data_file = get_data_file(config)

    if data_file.exists():
        try:
            with data_file.open("r", encoding="utf-8") as f:
                data = json.load(f)
        except (json.JSONDecodeError, OSError):
            data = {}
    else:
        data = {}

    data[key] = value

    with data_file.open("w", encoding="utf-8") as f:
        json.dump(data, f, indent=4)

def get_profiles(config: MWUtilConfig) -> list[str]:
    profiles = get_data_entry(config, "profiles", [])
    if "mysql" not in profiles and "mariadb" not in profiles:
        profiles.append(config.dbtype.db_name)
        save_profiles(config, profiles)
    return profiles

def save_profiles(config: MWUtilConfig, profiles: list[str]):
    set_data_entry(config, "profiles", profiles)

def disable_profile(config: MWUtilConfig, profile: str):
    profiles = get_profiles(config)
    if profile in profiles:
        profiles.remove(profile)
        save_profiles(config, profiles)

def enable_profile(config: MWUtilConfig, profile: str):
    profiles = get_profiles(config)
    if profile not in profiles:
        profiles.append(profile)
        save_profiles(config, profiles)
