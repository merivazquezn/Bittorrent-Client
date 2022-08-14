import React from 'react';
import logo from './logo.svg';
import './App.css';
import { Line } from '@ant-design/charts';
import { Button, Select, InputNumber, Typography, Input } from "antd";
import { message, Space } from 'antd';
import { ReloadOutlined } from '@ant-design/icons';
import { API } from "./API";

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
  key?: string;
}
const { Text } = Typography;


function App() {
  const data = [
    { moment: '1991', value: 10 },
    { moment: '1992', value: 4 },
    { moment: '1993', value: 3.5 },
    { moment: '1994', value: 5 },
    { moment: '1995', value: 4.9 },
    { moment: '1996', value: 6 },
    { moment: '1997', value: 7 },
    { moment: '1998', value: 35 },
    { moment: '1999', value: 13 },
  ];

  const [chartData, setChartData] = React.useState(data);

  const [config, setconfig] = React.useState<Config>({
    timeFrameInterval: 'hours',
    timeFrameCount: 1,
    groupBy: 'minutes',
    groupByCount: 1,
  });

  const chartConfig = {
    data: chartData,
    height: 400,
    width: document.documentElement.clientWidth * 0.8,
    xField: 'moment',
    yField: 'value',
    point: {
      size: 5,
      shape: 'diamond',
    },
    smooth: true
  };

  const handleReload = async () => {
    api.get("metrics", config).then(res => {
      console.log(res);
      setChartData(res.data);
    })
      .catch(err => {
        console.log(err);
        message.error('There was an error getting data from the server. Make sure you input valid parameters');

      }
      );
  }

  return (
    <div className="App">

      {/* <img style={{ position: "absolute", top: 20, left: 20 }} src={logo} className="App-logo" alt="logo" /> */}
      <header className="App-header">
        <div style={{ height: 50, width: document.documentElement.clientWidth * 0.8, display: "flex", justifyContent: "end", alignItems: "center" }}>
          <Text style={{ marginRight: 10, color: "white", fontWeight: "bold", fontSize: 15 }}>Metric</Text>
          <Input
            style={{ width: 250, marginRight: 25, fontWeight: 600, backgroundColor: "#444766", color: "white" }}
            value={config.key}
            placeholder="Example: torrents"
            onChange={(e) => {
              setconfig({ ...config, key: e.target.value })
            }} >
          </Input>
          <Text style={{ marginRight: 10, color: "white", fontWeight: "bold", fontSize: 15 }}>From Last</Text>
          <InputNumber
            style={{ width: 50, height: 32, marginRight: 5, fontWeight: 600, backgroundColor: "#444766", color: "white" }}
            value={config.timeFrameCount}
            onChange={(value) => {
              setconfig({ ...config, timeFrameCount: value })
            }
            }
          />
          <Select
            placeholder="Select a time frame interval"
            style={{ marginRight: 25, width: 230, fontWeight: 600, backgroundColor: "#444766", color: "white" }}
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
            onChange={(value) => {
              setconfig({ ...config, groupByCount: value })
            }
            }
          />
          <Select
            value={config.groupBy}
            style={{ marginRight: 25, width: 280 }}
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
