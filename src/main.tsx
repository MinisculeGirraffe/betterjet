import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.tsx'
import '@mantine/core/styles.css';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MantineProvider } from '@mantine/core'
import { HashProvider } from './context/HashContext.tsx';

const queryClient = new QueryClient()


ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <MantineProvider>
      <HashProvider>
        <QueryClientProvider client={queryClient}>
          <App />
        </QueryClientProvider>
      </HashProvider>
    </MantineProvider>
  </React.StrictMode>,
)
