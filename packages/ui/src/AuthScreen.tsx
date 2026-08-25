import { useState, type FormEvent } from "react";
import { Button } from "./Button";
import { Disclaimer } from "./Disclaimer";
import { Logo } from "./Logo";
import { useSession } from "./session";
import { ThemeSwitch } from "./ThemeSwitch";

export function AuthScreen() {
  const { api, setSession } = useSession();
  const [mode, setMode] = useState<"login" | "register">("login");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const result =
        mode === "login"
          ? await api.login({ email, password })
          : await api.register({ email, password, display_name: displayName || email });
      setSession(result.token, result.user);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Could not sign in");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="hii-auth">
      <div className="hii-auth-card">
        <Logo />
        <h1>Your health history, privately held</h1>
        <p className="hii-lede">Sign in to add measurements, labs and documents.</p>
        <form className="hii-form" onSubmit={onSubmit}>
          {mode === "register" ? (
            <label>
              Display name
              <input value={displayName} onChange={(event) => setDisplayName(event.target.value)} />
            </label>
          ) : null}
          <label>
            Email
            <input
              type="email"
              autoComplete="username"
              required
              value={email}
              onChange={(event) => setEmail(event.target.value)}
            />
          </label>
          <label>
            Password
            <input
              type="password"
              autoComplete={mode === "login" ? "current-password" : "new-password"}
              required
              minLength={10}
              value={password}
              onChange={(event) => setPassword(event.target.value)}
            />
          </label>
          {error ? <p className="hii-error">{error}</p> : null}
          <Button type="submit" disabled={busy}>
            {mode === "login" ? "Sign in" : "Create account"}
          </Button>
        </form>
        <Button
          variant="secondary"
          type="button"
          onClick={() => setMode(mode === "login" ? "register" : "login")}
        >
          {mode === "login" ? "Need an account?" : "Already have an account?"}
        </Button>
        <h2>Appearance</h2>
        <ThemeSwitch />
        <Disclaimer />
      </div>
    </div>
  );
}
