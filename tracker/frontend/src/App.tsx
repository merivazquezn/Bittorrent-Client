import React from 'react';
import logo from './logo.svg';
import './App.css';
import { Line } from '@ant-design/charts';
import { Button, Select, InputNumber, Typography, Tag } from "antd";
import { message, Space } from 'antd';
import { ReloadOutlined } from '@ant-design/icons';
import { API } from "./API";
import type { CustomTagProps } from 'rc-select/lib/BaseSelect';

function useInterval(callback: any, delay: any) {
  const savedCallback = React.useRef();

  React.useEffect(() => {
    savedCallback.current = callback;
  }, [callback]);

  React.useEffect(() => {
    function tick() {
      (savedCallback as any).current();
    }

    let id = setInterval(tick, delay);
    return () => clearInterval(id);
  }, [delay]);
}

message.config({
  top: 50,
  duration: 5,
  maxCount: 3,
  rtl: true,
});

const api = new API();

interface Config {
  timeFrameInterval?: 'hours' | 'days';
  timeFrameCount: number;
  groupBy?: 'hours' | 'minutes';
  groupByCount: number;
  keys: string[];
}
const { Text } = Typography;


function App() {

  const [chartData, setChartData] = React.useState<any[]>([]);
  const [torrents, setTorrents] = React.useState([""]);
  const [metric, setMetric] = React.useState("");
  const [cronRunning, setCronRunning] = React.useState(false);
  const [selectedTorrents, setSelectedTorrents] = React.useState<string[]>([]);
  const [config, setconfig] = React.useState<Config>({
    timeFrameInterval: 'hours',
    timeFrameCount: 1,
    groupBy: 'minutes',
    groupByCount: 1,
    keys: []
  });


  const tagRender = (props: CustomTagProps) => {
    const { label, value, closable, onClose } = props;
    const onPreventMouseDown = (event: React.MouseEvent<HTMLSpanElement>) => {
      event.preventDefault();
      event.stopPropagation();
    };
    return (
      <Tag
        color={"blue"}
        onMouseDown={onPreventMouseDown}
        closable={closable}
        onClose={onClose}
        style={{ marginRight: 3 }}
      >
        {label}
      </Tag>
    );
  };

  const updateTorrentList = () => {
    api.get("torrents").then(res => {
      console.log(res);
      setTorrents(res.data);
    })
      .catch(err => {
        console.log(err);
        message.error('error fetching torrents');
      });
  }

  const updateChartData = React.useCallback(async () => {
    if (config.keys.length === 0) {
      return;
    }

    // let cfg = { ...config, key: config.keys[0] };


    // try {
    //   let res = await api.get("metrics", cfg);
    //   console.log(res);
    //   setChartData(res.data);
    // } catch (err) {
    //   console.log(err);
    //   message.error('There was an error getting data from the server. Make sure you input valid parameters');

    // }

    const myPromiseArray = config.keys.map(key => {
      let cfg = { ...config, key };
      return api.get("metrics", cfg);
    })

    Promise.all(myPromiseArray)
      .then(res => {
        let newChartData: any[] = []
        res.map((timeSeriesOfTorrent: any, i: number) => {
          let timeSeriesWithKey = timeSeriesOfTorrent.data.map((timeValue: any) => ({ ...timeValue, metric: config.keys[i] }));
          newChartData = newChartData.concat(timeSeriesWithKey);
        })
        console.log("new chart data", newChartData);
        setChartData(newChartData);
      })
      .catch(err => {
        console.log(err);
        message.error("failed to get chart data");
      });

  }, [config]);



  const handleReload = React.useCallback(() => {
    if (metric === "torrents") {
      setconfig(cfg => ({ ...cfg, keys: ["torrents"] }))
    } else {
      let keys = selectedTorrents.map(torrent => torrent + "." + metric);
      setconfig(cfg => ({ ...cfg, keys }))
    }
  }, [metric, selectedTorrents]);


  // load torrent lists on start and every 30 seconds
  useInterval(() => {
    updateTorrentList();
    updateChartData();
  }, 1000 * 5);

  // update metrics if config changes
  React.useEffect(() => {
    updateChartData();
  }, [config, updateChartData])


  const metrics = [
    { key: "torrents", name: "Torrents" },
    { key: "active_peers", name: "Active peers" },
    { key: "complete_download_peers", name: "Downloads" },
  ]

  const chartConfig = {
    data: chartData,
    height: 400,
    width: document.documentElement.clientWidth * 0.8,
    xField: 'moment',
    yField: 'value',
    seriesField: 'metric',
    point: {
      size: 5,
      shape: 'diamond',
    },
    smooth: true
  };

  return (
    <div className="App">

      {/* <img style={{ position: "absolute", top: 20, left: 20 }} src={logo} className="App-logo" alt="logo" /> */}
      <header className="App-header">
        <div style={{ height: 50, width: document.documentElement.clientWidth * 0.8, display: "flex", justifyContent: "center", alignItems: "center" }}>
          <Text style={{ marginRight: 10, color: "white", fontWeight: "bold", fontSize: 15 }}>Metric</Text>
          <Select
            placeholder="Select a metric"
            style={{ marginRight: 25, width: 230, fontWeight: 600, backgroundColor: "#444766", color: "white" }}
            onChange={(value) => {
              setMetric(value);
            }}
          >
            {metrics.map(metric => <Select.Option key={metric.key} value={metric.key}>{metric.name}</Select.Option>
            )}
          </Select>
          {
            metric && metric !== "torrents" && (<>
              <Text style={{ marginRight: 10, color: "white", fontWeight: "bold", fontSize: 15 }}>Torrent</Text>
              <Select
                mode="multiple"
                placeholder="Select a torrent"
                value={selectedTorrents}
                notFoundContent={null}
                tagRender={tagRender}
                maxTagTextLength={8}
                onChange={setSelectedTorrents}
                style={{ width: 200, marginRight: 15 }}
              >
                {torrents.filter(o => !selectedTorrents.includes(o)).map(item => (
                  <Select.Option key={item} value={item}>
                    {item.slice(0, 8) + "..."}
                  </Select.Option>
                ))}
              </Select>
            </>
            )
          }
          <Text style={{ marginRight: 10, color: "white", fontWeight: "bold", fontSize: 15 }}>From Last</Text>
          <InputNumber
            style={{ width: 50, height: 32, marginRight: 5, fontWeight: 600, backgroundColor: "#444766", color: "white" }}
            value={config.timeFrameCount}
            min={1}
            onChange={(value) => {
              setconfig({ ...config, timeFrameCount: value })
            }
            }
          />
          <Select
            placeholder="Select a time frame interval"
            style={{ marginRight: 25, width: 100, fontWeight: 600, backgroundColor: "#444766", color: "white" }}
            value={config.timeFrameInterval}
            onChange={(value) => {
              setconfig({ ...config, timeFrameInterval: value })
            }
            }
          >
            <Select.Option value="hours">Hours</Select.Option>
            <Select.Option value="days">Days</Select.Option>
          </Select>


          <Text style={{ marginRight: 10, color: "white", fontWeight: "bold", fontSize: 15 }}>Group By</Text>

          <InputNumber
            style={{ width: 50, height: 32, marginRight: 5, fontWeight: 600, backgroundColor: "#444766", color: "white" }}
            value={config.groupByCount}
            min={1}
            onChange={(value) => {
              setconfig({ ...config, groupByCount: value })
            }
            }
          />
          <Select
            value={config.groupBy}
            style={{ marginRight: 25, width: 120 }}
            placeholder="Select a time interval to group by"

            onChange={(value) => {
              setconfig({ ...config, groupBy: value })
            }
            }
          >
            <Select.Option value="hours">Hours</Select.Option>
            <Select.Option value="minutes">Minutes</Select.Option>
          </Select>
          <Button type="primary" size='large' shape="circle" icon={<ReloadOutlined />} onClick={handleReload} />
        </div>
        <div style={{ marginTop: 20 }}>
          <Line {...chartConfig} />
        </div>
      </header >

    </div >
  );
}

export default App;
