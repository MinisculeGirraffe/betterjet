import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  get_adapters,
  get_config,
  get_status,
  scan_devices,
} from "../commands";
import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import {
  Event,
  EventCallback,
  listen,
  UnlistenFn,
} from "@tauri-apps/api/event";
import { DeviceEvent, ParsedDeviceStatus, PeripheralResult } from "../types";
import { usePrevious } from "@mantine/hooks";

export function useAdapters() {
  return useQuery({
    queryKey: ["adapters"],
    queryFn: () => get_adapters(),
    staleTime: Infinity,
  });
}

export function useDevices() {
  const adapters = useAdapters();
  const selectedAdapter = adapters.data?.selected;
  const enabled = !!selectedAdapter;

  useDeviceListSubscription();
  return useQuery({
    queryKey: ["devices"],
    queryFn: () => scan_devices(),
    enabled,
  });
}

export function useDeviceStatus(
  id: string | null | undefined,
) {
  useDeviceStatusSubscription(id ?? "");
  return useQuery({
    queryKey: ["devices", id],
    queryFn: () => get_status(id ?? ""),
    enabled: !!id,
  });
}

export function useSubscription<T>(
  id: string,
  onEvent: EventCallback<T>,
  shouldListen: boolean = true,
) {
  const [isListening, setIsListening] = useState(false);
  const isListeningRef = useRef(isListening); // Using ref to track value without causing re-renders

  // Synchronize the ref value with the state value
  useEffect(() => {
    isListeningRef.current = isListening;
  }, [isListening]);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    async function fetchAndListen() {
      unlisten = await listen<T>(id, onEvent);
      console.log("listening");
    }

    if (!isListeningRef.current && shouldListen) {
      setIsListening(true);
      fetchAndListen();
    }

    if (isListeningRef.current && !shouldListen && unlisten) {
      setIsListening(false);
      unlisten();
    }

    return () => {
      if (unlisten) unlisten();
    };
  }, [id, onEvent, shouldListen]); // Not dependent on isListening

  return { isListening };
}

function useDeviceListSubscription() {
  const queryClient = useQueryClient();

  const handleEvent = useCallback((event: Event<DeviceEvent>) => {
    console.log("DeviceEvent", event);
    queryClient.setQueryData<PeripheralResult[]>(
      ["devices"],
      (cache) => {
        if (cache === undefined) {
          return [event.payload.value];
        }
        const update = [...cache].map((i) =>
          i.id === event.payload.value.id
            ? { ...event.payload.value }
            : { ...i }
        );

        console.log({ cache, update });
        return update;
      },
    );
  }, [queryClient]);

  useSubscription<DeviceEvent>("DeviceEvent", handleEvent);
}

function useDeviceStatusSubscription(id: string) {
  const queryClient = useQueryClient();
  const handleEvent = useCallback((event: Event<ParsedDeviceStatus>) => {
    //console.log("DeviceStatusEvent", event);
    queryClient.setQueryData<ParsedDeviceStatus>(
      ["devices", id],
      event.payload,
    );
  }, [queryClient, id]);

  const subscription = useSubscription<ParsedDeviceStatus>(
    id,
    handleEvent,
    !!id,
  );

  return subscription;
}

export function useSyncedState<T>(
  currentValue: T,
  defaultValue: T,
  dependency: string,
): [T, React.Dispatch<React.SetStateAction<T>>] {
  const [value, setValue] = useState<T>(currentValue ?? defaultValue);

  const prevValue = usePrevious(currentValue);
  const prevDependency = usePrevious(dependency);

  useLayoutEffect(() => {
    if (
      (prevDependency !== dependency) ||
      (prevValue === undefined && currentValue !== undefined)
    ) {
      setValue(currentValue ?? defaultValue);
    }
  }, [currentValue, dependency, prevValue, prevDependency, defaultValue]);

  return [value, setValue];
}

export function useConfig() {
  return useQuery({
    queryKey: ["config"],
    queryFn: () => get_config(),
  });
}