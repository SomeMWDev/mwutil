from mwutil.module import MWUtilModule

class UpdateSymlinks(MWUtilModule):

    # TODO remove dead symlinks

    def get_description(self):
        return "Create symlinks for extensions and skins"

    def populate_subparser(self, parser, config):
        pass

    def execute(self, config, args):
        extensions_folder = config.coredir / "extensions"
        skins_folder = config.coredir / "skins"
        for folder in config.basedir.iterdir():
            if not folder.is_dir():
                # not a folder
                continue
            name = folder.stem
            if name.startswith(".") and name != ".mw-config":
                # hidden folder
                continue
            if folder == config.coredir or folder == config.dumpdir:
                # don't symlink core or the dump directory
                continue

            extension_link = extensions_folder / name
            if not extension_link.exists():
                extension_link.symlink_to(folder)
                print(f"Created symlink: {extension_link}")
            skin_link = skins_folder / name
            if not skin_link.exists():
                skin_link.symlink_to(folder)
                print(f"Created symlink: {skin_link}")
