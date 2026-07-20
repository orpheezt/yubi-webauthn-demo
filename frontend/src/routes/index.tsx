import { createFileRoute } from '@tanstack/react-router'
import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle, CardFooter } from '#/components/ui/card'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { toast } from 'sonner'
import { startRegistration, startAuthentication } from '@simplewebauthn/browser'
import { Fingerprint, KeyRound, UserRoundPlus } from 'lucide-react'

export const Route = createFileRoute('/')({ component: Home })

function Home() {
  const [username, setUsername] = useState('')
  const [loading, setLoading] = useState(false)

  const handleRegister = async () => {
    if (!username) {
      toast.error('Please enter a username')
      return
    }
    setLoading(true)
    try {
      toast.info('Starting registration...')
      
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
      
      toast.success('Registration successful!')
    } catch (err: any) {
      if (err.name === 'NotAllowedError') {
        toast.error('Registration cancelled by user')
      } else {
        toast.error(err.message || 'Registration failed')
      }
    } finally {
      setLoading(false)
    }
  }

  const handleLogin = async () => {
    setLoading(true)
    try {
      toast.info('Starting authentication...')
      
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
      
      toast.success('Authentication successful! Welcome back.')
    } catch (err: any) {
      if (err.name === 'NotAllowedError') {
        toast.error('Authentication cancelled by user')
      } else {
        toast.error(err.message || 'Authentication failed')
      }
    } finally {
      setLoading(false)
    }
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
            <div className="space-y-2">
              <Label htmlFor="username">Username</Label>
              <Input
                id="username"
                placeholder="alice@example.com"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                disabled={loading}
              />
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <Button 
                onClick={handleRegister} 
                disabled={loading}
                className="w-full bg-blue-600 hover:bg-blue-700 text-white"
              >
                <UserRoundPlus className="w-4 h-4 mr-2" />
                Register
              </Button>
              <Button 
                onClick={handleLogin} 
                disabled={loading}
                variant="secondary"
                className="w-full"
              >
                <KeyRound className="w-4 h-4 mr-2" />
                Authenticate
              </Button>
            </div>
          </CardContent>
          <CardFooter className="text-sm text-center text-slate-500 dark:text-slate-400 block pb-6">
            Try using your device's built-in authenticator or a security key.
          </CardFooter>
        </Card>

      </div>
    </div>
  )
}
