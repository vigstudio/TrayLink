import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { DevBanner } from "@/components/DevBanner";
import { ServerStatus } from "@/components/ServerStatus";
import { RequestLog } from "@/components/RequestLog";
import { AllowlistEditor } from "@/components/AllowlistEditor";
import { SettingsPanel } from "@/components/SettingsPanel";

function App() {
  return (
    <div className="min-h-screen bg-background">
      <DevBanner />
      <header className="border-b px-6 py-4">
        <h1 className="text-2xl font-semibold tracking-tight">TrayLink</h1>
        <p className="text-sm text-muted-foreground">
          App launcher chạy nền — HTTP API trên localhost
        </p>
      </header>

      <main className="p-6">
        <Tabs defaultValue="overview" className="space-y-4">
          <TabsList>
            <TabsTrigger value="overview">Overview</TabsTrigger>
            <TabsTrigger value="logs">Request Log</TabsTrigger>
            <TabsTrigger value="allowlist">Apps & Commands</TabsTrigger>
            <TabsTrigger value="settings">Settings</TabsTrigger>
          </TabsList>

          <TabsContent value="overview">
            <ServerStatus />
          </TabsContent>

          <TabsContent value="logs">
            <RequestLog />
          </TabsContent>

          <TabsContent value="allowlist">
            <AllowlistEditor />
          </TabsContent>

          <TabsContent value="settings">
            <SettingsPanel />
          </TabsContent>
        </Tabs>
      </main>
    </div>
  );
}

export default App;
