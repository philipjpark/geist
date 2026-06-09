import Image from "next/image";

export function GeistLogo({
  size = 40,
  className = "",
  priority = false,
}: {
  size?: number;
  className?: string;
  priority?: boolean;
}) {
  return (
    <Image
      src="/geist_logo1.png"
      alt="Geist"
      width={size}
      height={size}
      priority={priority}
      unoptimized
      className={`rounded-full object-cover ${className}`}
    />
  );
}
