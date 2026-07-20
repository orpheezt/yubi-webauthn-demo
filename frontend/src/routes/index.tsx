import { createFileRoute } from '@tanstack/react-router'
import { Card, CardContent, CardDescription, CardHeader, CardTitle, CardFooter } from '#/components/ui/card'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { toast } from 'sonner'
import { startRegistration, startAuthentication } from '@simplewebauthn/browser'
import { Fingerprint, KeyRound, UserRoundPlus } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { useMutation } from '@tanstack/react-query'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '#/components/ui/form'

export const Route = createFileRoute('/')({ component: Home })

const authSchema = z.object({
  username: z.string().email('Please enter a valid email address as your username.'),
})

function Home() {
  const form = useForm<z.infer<typeof authSchema>>({
    resolver: zodResolver(authSchema),
    defaultValues: {
      username: '',
    },
  })

  const registerMutation = useMutation({
    mutationFn: async (username: string) => {
      // Step 1: Start Registration
      const startRes = await fetch('/api/auth/register/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username })
      })
      if (!startRes.ok) throw new Error('Failed to start registration')
      const options = await startRes.json()
      
      // Step 2: Browser prompts for Passkey
      const attResp = await startRegistration({ optionsJSON: options })
      
      // Step 3: Finish Registration
      const finishRes = await fetch('/api/auth/register/finish', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(attResp)
      })
      if (!finishRes.ok) throw new Error('Registration verification failed')
    },
    onMutate: () => toast.info('Starting registration...'),
    onSuccess: () => toast.success('Registration successful!'),
    onError: (err: any) => {
      if (err.name === 'NotAllowedError') {
        toast.error('Registration cancelled by user')
      } else {
        toast.error(err.message || 'Registration failed')
      }
    }
  })

  const loginMutation = useMutation({
    mutationFn: async () => {
      // Step 1: Start Authentication (username not required for discoverable credentials)
      const startRes = await fetch('/api/auth/login/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username: '' })
      })
      if (!startRes.ok) throw new Error('Failed to start authentication')
      const options = await startRes.json()
      
      // Step 2: Browser prompts for Passkey
      const asseResp = await startAuthentication({ optionsJSON: options })
      
      // Step 3: Finish Authentication
      const finishRes = await fetch('/api/auth/login/finish', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(asseResp)
      })
      if (!finishRes.ok) throw new Error('Authentication verification failed')
    },
    onMutate: () => toast.info('Starting authentication...'),
    onSuccess: () => toast.success('Authentication successful! Welcome back.'),
    onError: (err: any) => {
      if (err.name === 'NotAllowedError') {
        toast.error('Authentication cancelled by user')
      } else {
        toast.error(err.message || 'Authentication failed')
      }
    }
  })

  const isLoading = registerMutation.isPending || loginMutation.isPending

  const onSubmitRegister = (values: z.infer<typeof authSchema>) => {
    registerMutation.mutate(values.username)
  }

  const onLogin = () => {
    loginMutation.mutate()
  }

  return (
    <div className="min-h-screen bg-slate-50 dark:bg-zinc-950 flex flex-col items-center justify-center p-4">
      <div className="w-full max-w-md space-y-6">
        
        {/* Header */}
        <div className="text-center space-y-2">
          <div className="mx-auto w-16 h-16 bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded-full flex items-center justify-center mb-4">
            <Fingerprint className="w-8 h-8" />
          </div>
          <h1 className="text-3xl font-bold tracking-tight text-slate-900 dark:text-slate-100">
            WebAuthn Demo
          </h1>
          <p className="text-slate-500 dark:text-slate-400">
            Passwordless authentication using Passkeys
          </p>
        </div>

        {/* Action Card */}
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
      </div>
    </div>
  )
}
