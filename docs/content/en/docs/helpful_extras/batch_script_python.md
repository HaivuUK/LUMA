---
date: '2026-06-12T15:54:46+01:00'
title: 'Python Batch Script'
category: 'Helpful Extras'
weight: 500
---

```python
import csv
import subprocess
import shlex
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed

# Path to the executable
EXE = Path(r"C:\users\username\appdata\local\name\name.exe")

# CSV file with columns: params,ct,mesh
# Example CSV rows:
# C:\path\params1.toml,C:\path\image1.vtk,C:\path\mesh1.cdb
# C:\path\params2.toml,C:\path\image2.vtk,C:\path\mesh2.cdb
CSV_INPUT = Path("batch_inputs.csv")

# Output directory for logs
OUT_DIR = Path("batch_logs")
OUT_DIR.mkdir(parents=True, exist_ok=True)

# How many runs to execute in parallel (set to 1 for sequential)
MAX_WORKERS = 4

def build_command(params_path: Path, ct_path: Path, mesh_path: Path) -> list:
    return [
        str(EXE),
        "--params", str(params_path),
        "--ct", str(ct_path),
        "--mesh", str(mesh_path),
    ]

def run_one(task_id: int, params: str, ct: str, mesh: str) -> dict:
    params_p = Path(params)
    ct_p = Path(ct)
    mesh_p = Path(mesh)

    cmd = build_command(params_p, ct_p, mesh_p)
    log_base = OUT_DIR / f"run_{task_id}"
    stdout_file = log_base.with_suffix(".out.txt")
    stderr_file = log_base.with_suffix(".err.txt")

    try:
        proc = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            check=False  # don't raise; capture returncode instead
        )
        stdout_file.write_text(proc.stdout)
        stderr_file.write_text(proc.stderr)
        return {
            "task_id": task_id,
            "cmd": " ".join(shlex.quote(p) for p in cmd),
            "returncode": proc.returncode,
            "stdout": str(stdout_file),
            "stderr": str(stderr_file),
        }
    except Exception as e:
        err_path = log_base.with_suffix(".exception.txt")
        err_path.write_text(str(e))
        return {
            "task_id": task_id,
            "cmd": " ".join(shlex.quote(p) for p in cmd),
            "returncode": None,
            "error": str(e),
            "exception_log": str(err_path),
        }

def read_tasks_from_csv(csv_path: Path) -> list:
    tasks = []
    with csv_path.open(newline="", encoding="utf-8") as f:
        reader = csv.reader(f)
        for i, row in enumerate(reader, start=1):
            if not row or row[0].strip().startswith("#"):
                continue
            # Expecting exactly three columns: params, ct, mesh
            if len(row) < 3:
                raise ValueError(f"CSV row {i} has fewer than 3 columns: {row}")
            tasks.append((i, row[0].strip(), row[1].strip(), row[2].strip()))
    return tasks

def main():
    tasks = read_tasks_from_csv(CSV_INPUT)
    results = []

    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as ex:
        futures = {ex.submit(run_one, tid, p, c, m): tid for tid, p, c, m in tasks}
        for fut in as_completed(futures):
            res = fut.result()
            results.append(res)
            status = res.get("returncode")
            if status == 0:
                print(f"Task {res['task_id']} succeeded (log: {res['stdout']})")
            else:
                print(f"Task {res['task_id']} failed (rc={status}) — see {res.get('stderr') or res.get('exception_log')}")

    # Optionally write a summary CSV
    summary_csv = OUT_DIR / "summary.csv"
    with summary_csv.open("w", newline="", encoding="utf-8") as f:
        writer = csv.writer(f)
        writer.writerow(["task_id", "cmd", "returncode", "stdout_log", "stderr_log", "error_or_exception"])
        for r in results:
            writer.writerow([
                r.get("task_id"),
                r.get("cmd"),
                r.get("returncode"),
                r.get("stdout", ""),
                r.get("stderr", ""),
                r.get("error") or r.get("exception_log", "")
            ])
    print(f"Finished {len(results)} tasks. Summary: {summary_csv}")

if __name__ == "__main__":
    main()
```
