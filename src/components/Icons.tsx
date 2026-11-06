type IconName = "plus" | "send" | "message" | "alert" | "close" | "arrow" | "session";

const paths: Record<IconName, string> = {
  plus: "M12 5v14M5 12h14",
  send: "M12 19V5m-6 6 6-6 6 6",
  message: "M20 11.5a7.5 7.5 0 0 1-7.5 7.5H5l-3 3V11.5A7.5 7.5 0 0 1 9.5 4H16a4 4 0 0 1 4 4v3.5Z",
  alert: "M12 8v5m0 3v.01M10.3 3.8l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.7-3.2l-8-14a2 2 0 0 0-3.4 0Z",
  close: "m6 6 12 12M6 18 18 6",
  arrow: "M5 12h14m-5-5 5 5-5 5",
  session: "M12 8v4l3 2M5.5 5.5A9 9 0 1 1 3 12M3 4v5h5",
};

export function Icon({ name, className = "" }: { name: IconName; className?: string }) {
  return <svg className={className} width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d={paths[name]} /></svg>;
}

