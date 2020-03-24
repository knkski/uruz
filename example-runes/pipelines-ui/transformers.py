from zipfile import ZipFile


def private_key(certificates: ZipFile, https: bool, port: int):
    """Returns privkey.pem from the certificates bundle."""

    if https:
        return certificates.read('/privkey.pem')


def full_chain(certificates: ZipFile, https: bool, port: int):
    """Returns fullchain.pem from the certificates bundle."""

    if https:
        return certificates.read('/fullchain.pem')
