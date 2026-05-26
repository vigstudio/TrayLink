import { useState } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { DevBanner } from "@/components/DevBanner";
import { ServerStatus } from "@/components/ServerStatus";
import { RequestLog } from "@/components/RequestLog";
import { AllowlistEditor } from "@/components/AllowlistEditor";
import { RemoteDeckEditor } from "@/components/RemoteDeckEditor";
import { SettingsPanel } from "@/components/SettingsPanel";

function App() {
  const [activeTab, setActiveTab] = useState("allowlist");

  return (
    <div className="min-h-screen bg-background">
      <DevBanner />
      <header className="border-b px-6 py-4">
        <div className="flex items-center gap-3">
          <img
            src="/icon.png"
            alt="TrayLink"
            className="h-10 w-10 rounded-lg object-cover"
          />
          <div>
            <h1 className="text-2xl font-semibold tracking-tight">TrayLink</h1>
            <p className="text-sm text-muted-foreground">
              App launcher chạy nền — HTTP API trên localhost
            </p>
          </div>
        </div>
        <p className="mt-1 text-sm text-muted-foreground">
          Source code:{" "}
          <a
            href="https://github.com/PhamMinhKha/TrayLink"
            target="_blank"
            rel="noreferrer"
            className="text-foreground underline underline-offset-4 hover:text-primary"
          >
            github.com/PhamMinhKha/TrayLink
          </a>
        </p>
      </header>

      <main className="p-6">
        <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-4">
          <TabsList>
            <TabsTrigger value="allowlist">Apps & Commands</TabsTrigger>
            <TabsTrigger value="remote">Remote Deck</TabsTrigger>
            <TabsTrigger value="overview">Overview</TabsTrigger>
            <TabsTrigger value="logs">Request Log</TabsTrigger>
            <TabsTrigger value="settings">Settings</TabsTrigger>
          </TabsList>

          <TabsContent value="allowlist">
            <AllowlistEditor />
          </TabsContent>

          <TabsContent value="remote" forceMount className="data-[state=inactive]:hidden">
            <RemoteDeckEditor active={activeTab === "remote"} />
          </TabsContent>

          <TabsContent value="overview">
            <ServerStatus />
          </TabsContent>

          <TabsContent value="logs">
            <RequestLog />
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
