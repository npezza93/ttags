#!/usr/bin/env python3
"""Compare two release binaries on deterministic synthetic Ruby input.

Example (build both binaries with the same toolchain and --release --locked):
  python3 scripts/benchmark-parsing.py /tmp/ttags-main target/release/ttags
  python3 scripts/benchmark-parsing.py /tmp/ttags-main target/release/ttags --delegate --counts 3000

Each pair uses the same corpus, file-list input, output location and Rayon
thread count. Order alternates between repetitions. Output bytes must match.
Synthetic timings do not predict a particular real repository's speedup.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import statistics
import subprocess
import tempfile
import time

SAMPLE = '''module Group{group:03d}
  class Record{number:04d} < ApplicationRecord
    FORMAT_VERSION = 1
    attr_accessor :title, :status
    attr_reader :identifier
    belongs_to :account
    has_many :comments
    scope :active, -> {{ where(active: true) }}
{delegate}
    def initialize(title = "untitled")
      @title = title
      @status = :draft
    end
    def self.fetch(identifier)
      find(identifier)
    end
{methods}
  end
end
'''
METHODS = "\n".join(
    f'''    def field_{index}(value = nil)
      result = value || @title
      result.to_s.strip
    end'''
    for index in range(10)
)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--counts", default="0,1,64,65,128,129,3100")
    parser.add_argument("--runs", type=int, default=3)
    parser.add_argument("--threads", type=int, default=8)
    parser.add_argument("--delegate", action="store_true")
    args = parser.parse_args()
    counts = [int(count) for count in args.counts.split(",")]
    if not counts or min(counts) < 0 or args.runs < 1 or args.threads < 1:
        parser.error("counts must be nonnegative; runs and threads must be positive")
    binaries = [("baseline", args.baseline.resolve()), ("candidate", args.candidate.resolve())]
    env = dict(os.environ, RAYON_NUM_THREADS=str(args.threads))
    with tempfile.TemporaryDirectory(prefix="ttags-parsing-benchmark-") as directory:
        root = Path(directory)
        paths = []
        for number in range(max(counts)):
            path = f"record_{number:04d}.rb"
            (root / path).write_text(SAMPLE.format(
                group=number // 100, number=number, methods=METHODS,
                delegate="    delegate :display_name, to: :account, prefix: true" if args.delegate else "",
            ))
            paths.append(path)
        for count in counts:
            data = "\0".join(paths[:count]).encode()
            results = []
            for iteration in range(args.runs):
                order = binaries if iteration % 2 == 0 else list(reversed(binaries))
                for label, binary in order:
                    start = time.perf_counter()
                    process = subprocess.run(
                        [str(binary), "--null", "--files-from", "-", "--tag-file", "tags"],
                        cwd=root, input=data, capture_output=True, env=env, timeout=300,
                    )
                    elapsed = time.perf_counter() - start
                    if process.returncode or process.stdout or process.stderr:
                        raise RuntimeError((label, process.returncode, process.stdout, process.stderr))
                    output = (root / "tags").read_bytes()
                    result = dict(binary=label, files=count, delegate=args.delegate,
                                  threads=args.threads, iteration=iteration, seconds=elapsed,
                                  tags=sum(not line.startswith(b"!_") for line in output.splitlines()),
                                  sha256=hashlib.sha256(output).hexdigest())
                    results.append(result)
                    print(json.dumps(result), flush=True)
            if len({result["sha256"] for result in results}) != 1:
                raise RuntimeError(f"Output mismatch for {count} files")
            medians = {label: statistics.median(result["seconds"] for result in results if result["binary"] == label)
                       for label, _ in binaries}
            print(json.dumps(dict(files=count, median_seconds=medians,
                                  speedup=medians["baseline"] / medians["candidate"])), flush=True)


if __name__ == "__main__":
    main()
