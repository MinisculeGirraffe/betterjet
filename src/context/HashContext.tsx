import { createContext, useContext } from 'react';
import { useHash } from '@mantine/hooks';


type Route = "#Main" | "#Settings"

const isRoute = (route: string): route is Route => (route === "#Main" || route === "#Settings")

type HashContextType = {
    route: Route,
    setRoute: (route: Route) => void
}

const HashContext = createContext<HashContextType>({
    route: '#Main',
    setRoute: () => { }
});


type HashProviderProps = {
    children: React.ReactNode
}

export function HashProvider({ children }: HashProviderProps) {
    const [hash, setHash] = useHash();
    const route = isRoute(hash) ? hash : "#Main"
    return (
        <HashContext.Provider value={{
            route: route,
            setRoute: (route) => setHash(route)

        }}>
            {children}
        </HashContext.Provider>
    )
}

export const useHashContext = () => useContext(HashContext)