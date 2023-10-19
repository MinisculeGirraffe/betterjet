import { AppShell, Container, Space, Stack } from '@mantine/core';
import { useState } from 'react';
import { secondsToHHMM } from './util';
import TempSlider from './components/TempSlider';
import FanSlider from './components/FanSlider';
import { DeviceList } from './components/DeviceList';
import { ModeControl } from './components/ModeControl';
import { useDeviceStatus } from './hooks';


function App() {
  const [id, setId] = useState<string | null>(null);
  const status = useDeviceStatus(id);
  console.log(status.data)
  return (
    <AppShell>
      <Container>
        <DeviceList onChange={setId} value={id} />
        {!!id &&
          <Stack>
            <p>{secondsToHHMM(status.data?.remaining_duration ?? 0)}</p>
            <ModeControl bedjet={id} data={status.data} />
            <TempSlider bedjet={id} data={status.data} />
            <Space />
            <FanSlider bedjet={id} data={status.data} />
          </Stack>
        }
      </Container>
    </AppShell>

  )
}

export default App

