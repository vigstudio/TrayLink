import { useEffect, useState } from "react";
import { Label } from "@/components/ui/label";
import { supportsBrowserProfiles } from "@/lib/browser";
import { listBrowserProfiles, type BrowserProfile } from "@/lib/tauri";

interface BrowserProfileSelectProps {
  id: string;
  path: string;
  value?: string;
  onChange: (profileId: string | undefined) => void;
}

const selectClassName =
  "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50";

export function BrowserProfileSelect({
  id,
  path,
  value,
  onChange,
}: BrowserProfileSelectProps) {
  const [profiles, setProfiles] = useState<BrowserProfile[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const visible = supportsBrowserProfiles(path);

  useEffect(() => {
    if (!visible) {
      setProfiles([]);
      setError("");
      return;
    }

    let cancelled = false;
    setLoading(true);
    setError("");

    listBrowserProfiles(path)
      .then((items) => {
        if (cancelled) return;
        setProfiles(items);
      })
      .catch((err) => {
        if (cancelled) return;
        setProfiles([]);
        setError(String(err));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [path, visible]);

  useEffect(() => {
    if (!value || profiles.length === 0) return;
    if (!profiles.some((item) => item.id === value)) {
      onChange(undefined);
    }
  }, [profiles, value, onChange]);

  if (!visible) {
    return null;
  }

  return (
    <div className="space-y-2">
      <Label htmlFor={id}>Profile trình duyệt</Label>
      <select
        id={id}
        className={selectClassName}
        value={value ?? ""}
        disabled={loading}
        onChange={(event) => {
          const next = event.target.value.trim();
          onChange(next || undefined);
        }}
      >
        <option value="">{loading ? "Đang tải profile..." : "Mặc định"}</option>
        {profiles.map((profile) => (
          <option key={profile.id} value={profile.id}>
            {profile.name}
          </option>
        ))}
      </select>
      <p className="text-xs text-muted-foreground">
        Chọn profile Chrome, Edge, Firefox… để mở đúng tài khoản. Safari chưa hỗ trợ.
      </p>
      {error && <p className="text-xs text-destructive">{error}</p>}
      {!loading && profiles.length === 0 && !error && (
        <p className="text-xs text-muted-foreground">
          Không tìm thấy profile — app sẽ mở profile mặc định của trình duyệt.
        </p>
      )}
    </div>
  );
}
