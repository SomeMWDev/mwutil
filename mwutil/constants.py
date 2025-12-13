SUPPORTED_BRANCHES = [
    "master",
    "REL1_39",
    "REL1_43",
    "REL1_44",
    "REL1_45",
]

REPO_TYPES = [
    "extension",
    "skin",
    "service",
    "tool"
]

REPO_FOLDERS = [f"{t}s" for t in REPO_TYPES]

# https://www.mediawiki.org/wiki/Manual:$wgLegalTitleChars
LEGAL_TITLE_REGEX = r"^[ %!\"$&\'()*,\-.\/0-9:;=?@A-Z\\^_`a-z~\x80-\xFF+]+$"
