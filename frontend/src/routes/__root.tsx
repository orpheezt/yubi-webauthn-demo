import { HeadContent, Scripts, createRootRoute } from '@tanstack/react-router'
import { Toaster } from '#/components/ui/sonner'

import appCss from '../styles.css?url'

export const Route = createRootRoute({
  head: () => ({
    meta: [
      {
        charSet: 'utf-8',
      },
      {
        name: 'viewport',
        content: 'width=device-width, initial-scale=1',
      },
      {
        title: 'WebAuthn Testing Dashboard',
      },
    ],
    links: [
      {
        rel: 'stylesheet',
        href: appCss,
      },
    ],
  }),
  shellComponent: RootDocument,
})

import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from '#/components/theme-provider'

const queryClient = new QueryClient()

function RootDocument({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <head>
        <HeadContent />
      </head>
      <body>
        <ThemeProvider defaultTheme="system" storageKey="vite-ui-theme">
          <QueryClientProvider client={queryClient}>
            {children}
          </QueryClientProvider>
          <Toaster />
        </ThemeProvider>
        <Scripts />
      </body>
    </html>
  )
}
