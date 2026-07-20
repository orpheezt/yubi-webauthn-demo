import { createFileRoute } from '@tanstack/react-router'
import { Card, CardContent, CardDescription, CardHeader, CardTitle, CardFooter } from '#/components/ui/card'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { toast } from 'sonner'
import { startRegistration, startAuthentication } from '@simplewebauthn/browser'
import { Fingerprint, KeyRound, UserRoundPlus, LogOut, ShieldCheck, User, Calendar, Key } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '#/components/ui/form'

export const Route = createFileRoute('/')({ component: Home })

const authSchema = z.object({
  username: z.string().email('Please enter a valid email address as your username.'),
})

interface UserProfile {
  id: string
  username: string
  created_at: string
  credentials_count: number
}

function Home() {
  const queryClient = useQueryClient()

  const form = useForm<z.infer<typeof authSchema>>({
    resolver: zodResolver(authSchema),
    defaultValues: {
      username: '',
    },
  })

  // User Session Query
  const { data: user, isLoading: isUserLoading } = useQuery<UserProfile | null>({
    queryKey: ['user-me'],
    queryFn: async () => {
      const res = await fetch('/api/auth/me')
      if (res.status === 401) return null
      if (!res.ok) throw new Error('Failed to fetch user profile')
      return res.json()
    },
    retry: false,
  })

  const registerMutation = useMutation({
    mutationFn: async (username: string) => {
      const startRes = await fetch('/api/auth/register/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username })
      })
      if (!startRes.ok) throw new Error('Failed to start registration')
      const options = await startRes.json()
      
      const attResp = await startRegistration({ optionsJSON: options.publicKey || options })
      
      const finishRes = await fetch('/api/auth/register/finish', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(attResp)
      })
      if (!finishRes.ok) throw new Error('Registration verification failed')
    },
    onMutate: () => toast.info('Starting registration...'),
    onSuccess: () => {
      toast.success('Registration successful!')
      queryClient.invalidateQueries({ queryKey: ['user-me'] })
    },
    onError: (err: any) => {
      if (err.name === 'NotAllowedError') {
        toast.error('Registration cancelled by user')
      } else {
        toast.error(err.message || 'Registration failed')
      }
    }
  })

  const loginMutation = useMutation({
    mutationFn: async (username?: string) => {
      const startRes = await fetch('/api/auth/login/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username: username || '' })
      })
      if (!startRes.ok) throw new Error('Failed to start authentication')
      const options = await startRes.json()
      
      const asseResp = await startAuthentication({ optionsJSON: options.publicKey || options })
      
      const finishRes = await fetch('/api/auth/login/finish', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(asseResp)
      })
      if (!finishRes.ok) throw new Error('Authentication verification failed')
    },
    onMutate: () => toast.info('Starting authentication...'),
    onSuccess: () => {
      toast.success('Authentication successful! Welcome back.')
      queryClient.invalidateQueries({ queryKey: ['user-me'] })
    },
    onError: (err: any) => {
      if (err.name === 'NotAllowedError') {
        toast.error('Authentication cancelled by user')
      } else {
        toast.error(err.message || 'Authentication failed')
      }
    }
  })

  const logoutMutation = useMutation({
    mutationFn: async () => {
      const res = await fetch('/api/auth/logout', { method: 'POST' })
      if (!res.ok) throw new Error('Logout failed')
    },
    onSuccess: () => {
      toast.success('Logged out successfully')
      queryClient.setQueryData(['user-me'], null)
      form.reset()
    },
    onError: (err: any) => toast.error(err.message || 'Logout failed'),
  })

  const isLoading = registerMutation.isPending || loginMutation.isPending || logoutMutation.isPending

  const onSubmitRegister = (values: z.infer<typeof authSchema>) => {
    registerMutation.mutate(values.username)
  }

  const onLogin = () => {
    const username = form.getValues('username')
    loginMutation.mutate(username)
  }

  return (
    <div className="min-h-screen bg-slate-50 dark:bg-zinc-950 flex flex-col items-center justify-center p-4">
      <div className="w-full max-w-md space-y-6">
        
        {/* Header */}
        <div className="text-center space-y-2">
          <div className="mx-auto w-16 h-16 bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded-full flex items-center justify-center mb-4 shadow-sm">
            <Fingerprint className="w-8 h-8" />
          </div>
          <h1 className="text-3xl font-bold tracking-tight text-slate-900 dark:text-slate-100">
            WebAuthn Demo
          </h1>
          <p className="text-slate-500 dark:text-slate-400">
            Passwordless authentication using Passkeys
          </p>
        </div>

        {/* Dynamic Card: Profile when logged in, Auth Form when logged out */}
        {user ? (
          <Card className="border-0 shadow-xl shadow-slate-200/50 dark:shadow-none dark:bg-zinc-900 overflow-hidden">
            <CardHeader className="bg-blue-600 text-white p-6">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <div className="w-12 h-12 bg-white/20 rounded-full flex items-center justify-center text-white font-bold text-lg">
                    {user.username.slice(0, 2).toUpperCase()}
                  </div>
                  <div>
                    <CardTitle className="text-xl text-white">{user.username}</CardTitle>
                    <div className="flex items-center text-xs text-blue-100 mt-1">
                      <ShieldCheck className="w-3.5 h-3.5 mr-1" />
                      Authenticated via Passkey
                    </div>
                  </div>
                </div>
              </div>
            </CardHeader>

            <CardContent className="p-6 space-y-4">
              <div className="space-y-3 text-sm">
                <div className="p-3 rounded-lg bg-slate-100 dark:bg-zinc-800 space-y-1">
                  <div className="flex items-center text-slate-600 dark:text-slate-400 text-xs font-medium">
                    <User className="w-3.5 h-3.5 mr-1.5" />
                    User ID
                  </div>
                  <div className="font-mono text-xs text-slate-900 dark:text-slate-200 break-all select-all">
                    {user.id}
                  </div>
                </div>

                <div className="flex items-center justify-between p-3 rounded-lg bg-slate-100 dark:bg-zinc-800">
                  <div className="flex items-center text-slate-600 dark:text-slate-400">
                    <Key className="w-4 h-4 mr-2" />
                    Registered Passkeys
                  </div>
                  <span className="font-semibold text-slate-900 dark:text-slate-200">
                    {user.credentials_count}
                  </span>
                </div>

                <div className="flex items-center justify-between p-3 rounded-lg bg-slate-100 dark:bg-zinc-800">
                  <div className="flex items-center text-slate-600 dark:text-slate-400">
                    <Calendar className="w-4 h-4 mr-2" />
                    Member Since
                  </div>
                  <span className="text-slate-900 dark:text-slate-200">
                    {new Date(user.created_at).toLocaleDateString()}
                  </span>
                </div>
              </div>
            </CardContent>

            <CardFooter className="p-6 pt-0">
              <Button 
                onClick={() => logoutMutation.mutate()} 
                disabled={isLoading}
                variant="destructive"
                className="w-full bg-red-600 hover:bg-red-700 text-white"
              >
                <LogOut className="w-4 h-4 mr-2" />
                Sign Out
              </Button>
            </CardFooter>
          </Card>
        ) : (
          <Card className="border-0 shadow-xl shadow-slate-200/50 dark:shadow-none dark:bg-zinc-900">
            <CardHeader>
              <CardTitle>Get Started</CardTitle>
              <CardDescription>
                Register a new passkey or authenticate with an existing one.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <Form {...form}>
                <form onSubmit={form.handleSubmit(onSubmitRegister)} className="space-y-6">
                  <FormField
                    control={form.control}
                    name="username"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Username</FormLabel>
                        <FormControl>
                          <Input placeholder="alice@example.com" disabled={isLoading} {...field} />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <Button 
                      type="submit" 
                      disabled={isLoading}
                      className="w-full bg-blue-600 hover:bg-blue-700 text-white"
                    >
                      <UserRoundPlus className="w-4 h-4 mr-2" />
                      Register
                    </Button>
                    <Button 
                      type="button"
                      onClick={onLogin} 
                      disabled={isLoading}
                      variant="secondary"
                      className="w-full"
                    >
                      <KeyRound className="w-4 h-4 mr-2" />
                      Authenticate
                    </Button>
                  </div>
                </form>
              </Form>
            </CardContent>
            <CardFooter className="text-sm text-center text-slate-500 dark:text-slate-400 block pb-6">
              Try using your device's built-in authenticator or a security key.
            </CardFooter>
          </Card>
        )}
      </div>
    </div>
  )
}
