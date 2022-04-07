# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from collections import namedtuple
import hashlib
import json
import os
import subprocess
from mach.decorators import (
    CommandArgument,
    Command,
    SubCommand,
)

generated_bindings_paths = {
    # UniFFI Examples/Fixtures
    'third_party/rust/uniffi-example-geometry/src/geometry.udl': {
        'webidl': 'dom/chrome-webidl/GeometryScaffolding.webidl',
        'cpp-header': 'toolkit/components/uniffi-bindgen-gecko-js/fixtures/GeometryScaffolding.h',
        'cpp': 'toolkit/components/uniffi-bindgen-gecko-js/fixtures/GeometryScaffolding.cpp',
        'js': 'toolkit/components/uniffi-bindgen-gecko-js/fixtures/Geometry.jsm',
    },
    'third_party/rust/uniffi-example-arithmetic/src/arithmetic.udl': {
        'webidl': 'dom/chrome-webidl/ArithmeticScaffolding.webidl',
        'cpp-header': 'toolkit/components/uniffi-bindgen-gecko-js/fixtures/ArithmeticScaffolding.h',
        'cpp': 'toolkit/components/uniffi-bindgen-gecko-js/fixtures/ArithmeticScaffolding.cpp',
        'js': 'toolkit/components/uniffi-bindgen-gecko-js/fixtures/Arithmetic.jsm',
    },
    'third_party/rust/uniffi-example-rondpoint/src/rondpoint.udl': {
        'webidl': 'dom/chrome-webidl/RondpointScaffolding.webidl',
        'cpp-header': 'toolkit/components/uniffi-bindgen-gecko-js/fixtures/RondpointScaffolding.h',
        'cpp': 'toolkit/components/uniffi-bindgen-gecko-js/fixtures/RondpointScaffolding.cpp',
        'js': 'toolkit/components/uniffi-bindgen-gecko-js/fixtures/Rondpoint.jsm',
    },
}

def build_uniffi_bindgen_gecko_js(command_context):
    uniffi_root = crate_root(command_context)
    print("Building uniffi-bindgen-gecko-js")
    subprocess.check_call(['cargo', 'build', '--release'], cwd=uniffi_root)
    print()
    return os.path.join(command_context.topsrcdir, 'target', 'release', 'uniffi-bindgen-gecko-js')

@Command(
    "uniffi",
    category="devenv",
    description="Generate JS bindings using uniffi-bindgen-gecko-js",
)
def uniffi(command_context, *runargs, **lintargs):
    """Run linters."""
    command_context._sub_mach(["help", "uniffi"])
    return 1

@SubCommand(
    "uniffi",
    "generate",
    description="Generate/regenerate bindings",
)
def generate_command(command_context):
    binary_path = build_uniffi_bindgen_gecko_js(command_context)
    last_generated_hashes = load_hashes(command_context)
    uniffi_bindgen_hash = hash_directory(crate_root(command_context))
    if uniffi_bindgen_hash != last_generated_hashes.get('uniffi-bindgen'):
        # uniffi_bindgen source changed or no hashes loaded.  Start from scratch.
        last_generated_hashes = {
            'uniffi-bindgen': uniffi_bindgen_hash
        }


    for udl_path, generated_files in generated_bindings_paths.items():
        udl_hash = hash_file(udl_path)
        if udl_hash == last_generated_hashes.get(udl_path):
            print(f"{udl_path} not changed, skipping")
            continue
        else:
            last_generated_hashes[udl_path] = udl_hash
        for mode, path in generated_files.items():
            print(f"Generating {path} from {udl_path} ({mode} mode)")
            subprocess.check_call([binary_path, '--out', path, mode, udl_path])
    save_hashes(command_context, last_generated_hashes)
    return 0

@SubCommand(
    "uniffi",
    "print",
    description="Print generated binding to STOUT",
)
@CommandArgument(
    "filename",
    help="Filename to generate",
)
def print_command(command_context, filename):
    binary_path = build_uniffi_bindgen_gecko_js(command_context)
    for udl_path, generated_files in generated_bindings_paths.items():
        for mode, path in generated_files.items():
            if os.path.basename(path) == filename:
                print("\n{} {} {}".format("-" * 30, filename, "-" * 30))
                subprocess.check_call([binary_path, '--stdout', mode, udl_path])
                return 0
    print(f"Don't know how to generate {filename}")
    return 1

def load_hashes(command_context):
    path = hash_file_path(command_context)
    if not os.path.exists(path):
        return {}
    try:
        with open(path, 'r') as f:
            return json.load(f)
    except:
        print(f"Error loading hashes from {path}.  All files will be regenerated")
        return {}

def save_hashes(command_context, hashes):
    path = hash_file_path(command_context)
    with open(path, 'w') as f:
        return json.dump(hashes, f)

def hash_file(path):
    hasher = hashlib.sha256()
    with open(path, 'rb') as f:
        hasher.update(f.read())
    return hasher.hexdigest()

def hash_directory(path):
    hasher = hashlib.sha256()
    for root, _, files in os.walk(path):
        for filename in files:
            with open(os.path.join(root, filename), 'rb') as f:
                hasher.update(f.read())
    return hasher.hexdigest()

def hash_file_path(command_context):
    return os.path.join(command_context.topobjdir, "uniffi-bindgen-gecko-js.hashes")

def crate_root(command_context):
    return os.path.join(command_context.topsrcdir, 'toolkit', 'components', 'uniffi-bindgen-gecko-js')
