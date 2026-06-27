import appIconUrl from "../../assets/icon.png";

interface AppIconProps {
  size?: number;
  className?: string;
}

export function AppIcon({ size = 36, className = "" }: AppIconProps) {
  return (
    <img
      src={appIconUrl}
      alt=""
      width={size}
      height={size}
      draggable={false}
      className={["object-cover", className].filter(Boolean).join(" ")}
    />
  );
}