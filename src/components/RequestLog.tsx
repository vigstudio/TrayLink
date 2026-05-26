import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { getRequestLogs, type LogEntry } from "@/lib/tauri";

function statusVariant(status: number): "default" | "destructive" | "secondary" {
  if (status >= 500) return "destructive";
  if (status >= 400) return "secondary";
  return "default";
}

export function RequestLog() {
  const [logs, setLogs] = useState<LogEntry[]>([]);

  useEffect(() => {
    const load = async () => {
      const entries = await getRequestLogs();
      setLogs([...entries].reverse());
    };
    load();
    const interval = setInterval(load, 2000);
    return () => clearInterval(interval);
  }, []);

  return (
    <Card>
      <CardHeader>
        <CardTitle>Request Log</CardTitle>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-[420px]">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Thời gian</TableHead>
                <TableHead>Method</TableHead>
                <TableHead>Path</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Duration</TableHead>
                <TableHead>IP</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {logs.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={6} className="text-center text-muted-foreground">
                    Chưa có request nào
                  </TableCell>
                </TableRow>
              ) : (
                logs.map((log, index) => (
                  <TableRow key={`${log.timestamp}-${index}`}>
                    <TableCell className="whitespace-nowrap text-xs">
                      {new Date(log.timestamp).toLocaleString()}
                    </TableCell>
                    <TableCell>{log.method}</TableCell>
                    <TableCell>{log.path}</TableCell>
                    <TableCell>
                      <Badge variant={statusVariant(log.status)}>{log.status}</Badge>
                    </TableCell>
                    <TableCell>{log.duration_ms}ms</TableCell>
                    <TableCell>{log.client_ip}</TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
