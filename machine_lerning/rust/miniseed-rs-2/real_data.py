from obspy import read
import json
from pathlib import Path
def output_miniseed_file(file_path:str):
    file_path: Path = Path(file_path)

    wf_file = read(file_path)
    output_data = {"traces": []}
    for trace in wf_file:
        raw_data = []
        write_binary(trace, file_path.absolute().parent)
        for sample in trace.data:
            add_sample = int(sample)
            raw_data.append(add_sample)
        print(f"duration: {trace.stats.endtime - trace.stats.starttime}")
        print(f"sample rate: {trace.stats.sampling_rate}")
        print(trace.stats.npts)
        data = {
            "start_time": str(trace.stats.starttime),
            "end_time": str(trace.stats.endtime),
            "network": str(trace.stats.network),
            "station": str(trace.stats.station),
            "channel": str(trace.stats.channel),
            "sampling_rate": trace.stats.sampling_rate,
            "num_points": trace.stats.npts,
        }
        output_data["traces"].append(data)
        with open(file_path.absolute().parent / "metadata.json", "w") as f:
            json.dump(output_data,f)

def write_binary(trace, parent_path: Path):
    print(parent_path)
    save_path = parent_path / f"data_{trace.stats['channel']}.bin"
    with open(save_path,"b+w") as f:
        f.write(trace.data)
if __name__ == '__main__':
    output_miniseed_file("./raw_data/data.mseed")
