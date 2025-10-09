#!/usr/bin/env python3
"""
网络搜索缓存清理工具

这个脚本用于清理损坏的网络搜索配置缓存，解决 ProtoBuf 解析错误。

用法:
    python3 clear_web_search_cache.py [--backup]

选项:
    --backup    在清理前备份缓存文件
"""

import os
import sys
import shutil
import sqlite3
from pathlib import Path
from datetime import datetime
import argparse


def get_cache_db_paths():
    """获取缓存数据库文件路径"""
    home = Path.home()
    
    paths = []
    # 主要的缓存位置
    cache_path1 = home / "Documents" / "AppFlowyDataDoNotRename" / "cache.db"
    
    # 备用的缓存位置
    app_support = home / "Library" / "Application Support" / "com.appflowy.appflowy.flutter"
    cache_path2 = app_support / "data_beta.appflowy.cloud" / "cache.db"
    cache_path3 = app_support / "data" / "cache.db"
    
    if cache_path1.exists():
        paths.append(cache_path1)
    if cache_path2.exists():
        paths.append(cache_path2)
    if cache_path3.exists():
        paths.append(cache_path3)
    
    return paths


def backup_cache_file(cache_path: Path):
    """备份缓存文件"""
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    backup_dir = Path.home() / "Desktop" / "appflowy_cache_backup"
    backup_dir.mkdir(parents=True, exist_ok=True)
    
    backup_path = backup_dir / f"{cache_path.name}_{timestamp}.backup"
    
    try:
        shutil.copy2(cache_path, backup_path)
        print(f"✅ 已备份: {backup_path}")
        return True
    except Exception as e:
        print(f"❌ 备份失败: {e}")
        return False


def clear_web_search_keys(cache_path: Path):
    """清除数据库中的网络搜索相关键值"""
    try:
        conn = sqlite3.connect(str(cache_path))
        cursor = conn.cursor()
        
        # 查询需要删除的键
        cursor.execute("SELECT key FROM kv_table WHERE key LIKE '%web_search%'")
        keys_to_delete = cursor.fetchall()
        
        if not keys_to_delete:
            print(f"  ℹ️  未找到网络搜索相关的配置")
            conn.close()
            return True
        
        print(f"  📝 找到 {len(keys_to_delete)} 个网络搜索相关配置:")
        for (key,) in keys_to_delete:
            print(f"     - {key}")
        
        # 删除这些键
        cursor.execute("DELETE FROM kv_table WHERE key LIKE '%web_search%'")
        conn.commit()
        
        deleted_count = cursor.rowcount
        conn.close()
        
        print(f"  ✅ 已删除 {deleted_count} 个配置项")
        return True
        
    except Exception as e:
        print(f"  ❌ 清理失败: {e}")
        return False


def check_appflowy_running():
    """检查 AppFlowy 是否正在运行"""
    import subprocess
    try:
        result = subprocess.run(
            ["pgrep", "-f", "AppFlowy"],
            capture_output=True,
            text=True
        )
        return result.returncode == 0
    except Exception:
        return False


def main():
    parser = argparse.ArgumentParser(
        description="清理 AppFlowy 网络搜索配置缓存",
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        "--backup",
        action="store_true",
        help="在清理前备份缓存文件"
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="即使 AppFlowy 正在运行也强制清理（不推荐）"
    )
    
    args = parser.parse_args()
    
    print("=" * 60)
    print("🧹 AppFlowy 网络搜索缓存清理工具")
    print("=" * 60)
    print()
    
    # 检查应用是否在运行
    if check_appflowy_running() and not args.force:
        print("⚠️  检测到 AppFlowy 正在运行！")
        print("   请先关闭 AppFlowy 应用再运行此脚本。")
        print()
        print("   如果确定要继续，请使用 --force 参数:")
        print("   python3 clear_web_search_cache.py --force")
        return 1
    
    # 获取缓存文件路径
    cache_paths = get_cache_db_paths()
    
    if not cache_paths:
        print("❌ 未找到缓存数据库文件")
        print("   请确认 AppFlowy 已安装并至少运行过一次")
        return 1
    
    print(f"📂 找到 {len(cache_paths)} 个缓存文件:")
    for path in cache_paths:
        print(f"   - {path}")
    print()
    
    # 备份（如果需要）
    if args.backup:
        print("📦 开始备份...")
        for cache_path in cache_paths:
            backup_cache_file(cache_path)
        print()
    
    # 清理网络搜索配置
    print("🧹 开始清理网络搜索配置...")
    success_count = 0
    
    for cache_path in cache_paths:
        print(f"\n处理: {cache_path.name}")
        if clear_web_search_keys(cache_path):
            success_count += 1
    
    print()
    print("=" * 60)
    
    if success_count == len(cache_paths):
        print("✅ 清理完成！")
        print()
        print("📝 下一步:")
        print("   1. 重新启动 AppFlowy 应用")
        print("   2. 进入「设置」→「网络搜索」")
        print("   3. 检查是否可以正常加载")
        print()
        print("   如果问题仍然存在，请尝试:")
        print("   - 重新编译 Rust 库: cd rust-lib && cargo build")
        print("   - 重新构建应用: cd appflowy_flutter && flutter clean && flutter run")
        return 0
    else:
        print("⚠️  部分清理失败")
        print(f"   成功: {success_count}/{len(cache_paths)}")
        return 1


if __name__ == "__main__":
    sys.exit(main())

