import { ActionIcon, AppShell, Container, Group, Select, Space, Stack } from '@mantine/core';
import { useState } from 'react';
import { secondsToHHMM } from './util';
import TempSlider from './components/TempSlider';
import FanSlider from './components/FanSlider';
import { DeviceList } from './components/DeviceList';
import { ModeControl } from './components/ModeControl';
import { useDeviceStatus } from './hooks';
import { useHashContext } from './context/HashContext';
import { match } from 'ts-pattern';
import { IconSettings, IconHome } from '@tabler/icons-react';
import { BluetoothAdapterSelect } from './components/BluetoothAdapterSelect';


function MainPage({ id }: { id: string | null }) {
  const status = useDeviceStatus(id);
  return (
    <Container>
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
  )
}

function SettingsPage() {
  return (
    <Container>
      <BluetoothAdapterSelect />
      <Select label="Units" data={["Imperial", "Metric"]} allowDeselect={false} value={"Imperial"}></Select>
    </Container>
  )
}

function App() {
  const { route, setRoute } = useHashContext()
  const [id, setId] = useState<string | null>(null);

  return (
    <AppShell header={{"offset": true,height:35}}>
      <AppShell.Header>
        <Group >
          <DeviceList  onChange={setId} value={id} />
          {match(route)
            .with("#Main", () =>
              <ActionIcon variant="outline" color="gray" onClick={() => setRoute("#Settings")}>
                <IconSettings />
              </ActionIcon>
            )
            .with("#Settings", () =>
              <ActionIcon variant="outline" color="gray" onClick={() => setRoute("#Main")}>
                <IconHome />
              </ActionIcon>)
            .exhaustive()}
        </Group>
      </AppShell.Header>
      <AppShell.Main>
        {match(route)
          .with("#Main", () => <MainPage id={id} />)
          .with("#Settings", () => <SettingsPage />)
          .exhaustive()}
      </AppShell.Main>

    </AppShell>

  )
}

export default App


