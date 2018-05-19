# Tako

Tako: take container image.

Container runtimes are overrated. [Systemd can take care of the sandboxing
part][containers-systemd]. Tako takes care of versioned image acquisition
and automatic updates.

## Goals

Goals:

 * Securely downloading signed images.
 * Implement a versioning policy, to be able to download the latest compatible
   version of an image. Automatic security updates, but not new versions with
   breaking changes without manual intervention.

Non-goals:

 * Reinvent apt packaging. In particular: no scriptable install steps or
   extensive metadata. Just a signed filesystem image. Not even systemd unit
   files.
 * Be a container runtime. Systemd is a decent container runtime.

## Overview

Tako is a short-lived process that downloads images specified in its
configuration and then exits. Optionally Tako restarts configured systemd units
when it downloads a newer version of an image. Tako runs on two occasions:

 * Periodically, triggered by a systemd timer. Tako will check for new
   compatible versions of a configured image. If one exists, Tako downloads it
   and restarts the systemd unit that uses the image.
 * As a dependency of the systemd unit that uses the image, to provision a clean
   system with an initial image.

## Usage

Command-line interface:

    # Initially fetch an image, but do nothing if any image exists already.
    tako fetch --init /etc/tako/yourapp

    # Check for, download, and apply available updates.
    tako fetch /etc/tako/yourapp

    # Update multiple images at once.
    tako fetch /etc/tako/app-foo /etc/tako/app-bar

Configuration file example:

    Origin=https://images.example.com/app-foo
    PublicKey=8+r5DKNN/cwI+h0oHxMtgdyND3S/5xDLHQu0hFUmq+g=
    Destination=/var/lib/images/app-foo
    RestartUnit=app-foo.service
    Version=1.*

If multiple units share the same image, it is possible to specify multiple units
to restart:

    Origin=https://images.example.com/app-foo
    PublicKey=8+r5DKNN/cwI+h0oHxMtgdyND3S/5xDLHQu0hFUmq+g=
    Destination=/var/lib/images/app-foo
    RestartUnit=app-foo.service
    RestartUnit=app-bar.service
    Version=1.*

The `RestartUnit=` key is optional.

## Building

    cargo build --release
    target/release/tako --help

## Server

A Tako server is a regular http server, with a particular directory layout. The
origin uri points to a directory where we can find the manifest file that lists
all available versions and their SHA256 digests. The manifest is signed.
See also [Manifest Format](docs/manifest-format.md) in the docs.

## Local Store

Tako downloads images into a destination directory. It creates the following
files there:

    store/<hexdigest>  # Raw image files.
    manifest           # A copy of the manifest served by the origin.
    latest             # Symlink to the latest image.

## Future work

 * GC'ing the local store.
 * Differential updates. (Bsdiff, Casync?)

[containers-systemd]: https://media.ccc.de/v/ASG2017-101-containers_without_a_container_manager_with_systemd
