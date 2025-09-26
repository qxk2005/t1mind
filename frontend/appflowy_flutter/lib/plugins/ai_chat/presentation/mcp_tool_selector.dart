import 'dart:async';

import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/plugins/ai_chat/application/execution_log_entities.dart' as log_entities;
import 'package:flowy_infra/theme_extension.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flowy_infra_ui/style_widget/button.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flutter/material.dart';
import 'package:appflowy_popover/appflowy_popover.dart';

/// MCP工具选择器组件
/// 
/// 支持多选、过滤和状态显示的工具选择器，提供：
/// - 多选MCP工具
/// - 实时搜索和过滤
/// - 工具状态显示
/// - 分类显示
/// - 高效的大量工具显示
/// - 清晰的选择状态
class McpToolSelector extends StatefulWidget {
  const McpToolSelector({
    super.key,
    required this.availableTools,
    required this.selectedToolIds,
    required this.onSelectionChanged,
    this.maxHeight = 400,
    this.maxWidth = 320,
    this.enableSearch = true,
    this.enableCategoryFilter = true,
    this.showToolStatus = true,
    this.showToolStats = true,
    this.compactMode = false,
  });

  /// 可用的MCP工具列表
  final List<log_entities.McpToolInfo> availableTools;
  
  /// 当前选中的工具ID列表
  final List<String> selectedToolIds;
  
  /// 选择变更回调
  final ValueChanged<List<String>> onSelectionChanged;
  
  /// 最大高度
  final double maxHeight;
  
  /// 最大宽度
  final double maxWidth;
  
  /// 是否启用搜索功能
  final bool enableSearch;
  
  /// 是否启用分类过滤
  final bool enableCategoryFilter;
  
  /// 是否显示工具状态
  final bool showToolStatus;
  
  /// 是否显示工具统计信息
  final bool showToolStats;
  
  /// 是否使用紧凑模式
  final bool compactMode;

  @override
  State<McpToolSelector> createState() => _McpToolSelectorState();
}

class _McpToolSelectorState extends State<McpToolSelector> {
  final TextEditingController _searchController = TextEditingController();
  final PopoverController _popoverController = PopoverController();
  
  String _searchKeyword = '';
  String? _selectedCategory;
  List<String> _selectedToolIds = [];
  Timer? _searchDebouncer;
  
  // 缓存过滤后的工具列表以提高性能
  List<log_entities.McpToolInfo> _filteredTools = [];
  List<String> _availableCategories = [];

  @override
  void initState() {
    super.initState();
    _selectedToolIds = List.from(widget.selectedToolIds);
    _updateFilteredTools();
    _updateAvailableCategories();
  }

  @override
  void didUpdateWidget(McpToolSelector oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.availableTools != widget.availableTools ||
        oldWidget.selectedToolIds != widget.selectedToolIds) {
      _selectedToolIds = List.from(widget.selectedToolIds);
      _updateFilteredTools();
      _updateAvailableCategories();
    }
  }

  @override
  void dispose() {
    _searchController.dispose();
    _searchDebouncer?.cancel();
    super.dispose();
  }

  void _updateFilteredTools() {
    _filteredTools = widget.availableTools.where((tool) {
      // 搜索过滤
      if (_searchKeyword.isNotEmpty) {
        final keyword = _searchKeyword.toLowerCase();
        final matchesName = tool.name.toLowerCase().contains(keyword);
        final matchesDisplayName = 
            tool.displayName?.toLowerCase().contains(keyword) ?? false;
        final matchesDescription = tool.description.toLowerCase().contains(keyword);
        final matchesProvider = tool.provider.toLowerCase().contains(keyword);
        
        if (!matchesName && !matchesDisplayName && !matchesDescription && !matchesProvider) {
          return false;
        }
      }
      
      // 分类过滤
      if (_selectedCategory != null && _selectedCategory!.isNotEmpty) {
        if (tool.category != _selectedCategory) {
          return false;
        }
      }
      
      return true;
    }).toList();
    
    // 按状态和使用频率排序
    _filteredTools.sort((a, b) {
      // 首先按状态排序（可用的在前）
      final aAvailable = a.status.isAvailable ? 1 : 0;
      final bAvailable = b.status.isAvailable ? 1 : 0;
      final statusCompare = bAvailable.compareTo(aAvailable);
      if (statusCompare != 0) return statusCompare;
      
      // 然后按使用次数排序（使用多的在前）
      final usageCompare = b.usageCount.compareTo(a.usageCount);
      if (usageCompare != 0) return usageCompare;
      
      // 最后按名称排序
      return a.name.compareTo(b.name);
    });
  }

  void _updateAvailableCategories() {
    final categories = widget.availableTools
        .map((tool) => tool.category)
        .where((category) => category.isNotEmpty)
        .toSet()
        .toList();
    categories.sort();
    _availableCategories = categories;
  }

  void _onSearchChanged(String keyword) {
    _searchDebouncer?.cancel();
    _searchDebouncer = Timer(const Duration(milliseconds: 300), () {
      if (mounted) {
        setState(() {
          _searchKeyword = keyword;
          _updateFilteredTools();
        });
      }
    });
  }

  void _onCategoryChanged(String? category) {
    setState(() {
      _selectedCategory = category;
      _updateFilteredTools();
    });
  }

  void _onToolToggled(String toolId, bool selected) {
    setState(() {
      if (selected) {
        if (!_selectedToolIds.contains(toolId)) {
          _selectedToolIds.add(toolId);
        }
      } else {
        _selectedToolIds.remove(toolId);
      }
    });
    widget.onSelectionChanged(_selectedToolIds);
  }

  void _selectAll() {
    setState(() {
      _selectedToolIds = _filteredTools.map((tool) => tool.id).toList();
    });
    widget.onSelectionChanged(_selectedToolIds);
  }

  void _clearAll() {
    setState(() {
      _selectedToolIds.clear();
    });
    widget.onSelectionChanged(_selectedToolIds);
  }

  @override
  Widget build(BuildContext context) {
    final selectedCount = _selectedToolIds.length;
    final totalCount = widget.availableTools.length;
    
    return AppFlowyPopover(
      controller: _popoverController,
      direction: PopoverDirection.bottomWithLeftAligned,
      constraints: BoxConstraints(
        maxHeight: widget.maxHeight,
        maxWidth: widget.maxWidth,
        minWidth: 280,
      ),
      child: _buildTriggerButton(selectedCount, totalCount),
      popupBuilder: (context) => _buildPopoverContent(),
    );
  }

  Widget _buildTriggerButton(int selectedCount, int totalCount) {
    final hasSelection = selectedCount > 0;
    
    return FlowyButton(
      text: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          FlowySvg(
            FlowySvgs.add_s,
            size: const Size.square(16),
            color: hasSelection 
                ? Theme.of(context).colorScheme.primary
                : AFThemeExtension.of(context).textColor,
          ),
          const SizedBox(width: 8),
          FlowyText.regular(
            hasSelection 
                ? "已选择 $selectedCount/$totalCount 工具"
                : "选择MCP工具",
            fontSize: 14,
            color: hasSelection 
                ? Theme.of(context).colorScheme.primary
                : AFThemeExtension.of(context).textColor,
          ),
          const SizedBox(width: 4),
          Icon(
            Icons.arrow_drop_down,
            size: 16,
            color: AFThemeExtension.of(context).textColor,
          ),
        ],
      ),
      onTap: () => _popoverController.show(),
      hoverColor: AFThemeExtension.of(context).greyHover,
    );
  }

  Widget _buildPopoverContent() {
    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(8),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.1),
            blurRadius: 8,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          _buildHeader(),
          if (widget.enableSearch) _buildSearchBar(),
          if (widget.enableCategoryFilter && _availableCategories.isNotEmpty)
            _buildCategoryFilter(),
          _buildToolsList(),
          _buildFooter(),
        ],
      ),
    );
  }

  Widget _buildHeader() {
    final selectedCount = _selectedToolIds.length;
    final filteredCount = _filteredTools.length;
    
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        border: Border(
          bottom: BorderSide(
            color: AFThemeExtension.of(context).borderColor,
            width: 1,
          ),
        ),
      ),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                FlowyText.medium(
                  "选择MCP工具",
                  fontSize: 14,
                ),
                const SizedBox(height: 2),
                FlowyText.regular(
                  "已选择 $selectedCount 个工具，共 $filteredCount 个可用",
                  fontSize: 11,
                  color: AFThemeExtension.of(context).textColor,
                ),
              ],
            ),
          ),
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              FlowyButton(
                text: FlowyText.regular(
                  "全选",
                  fontSize: 11,
                  color: Theme.of(context).colorScheme.primary,
                ),
                onTap: _selectAll,
                hoverColor: Theme.of(context).colorScheme.primary.withOpacity(0.1),
              ),
              const SizedBox(width: 4),
              FlowyButton(
                text: FlowyText.regular(
                  "清空",
                  fontSize: 11,
                  color: AFThemeExtension.of(context).textColor,
                ),
                onTap: _clearAll,
                hoverColor: AFThemeExtension.of(context).greyHover,
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildSearchBar() {
    return Container(
      padding: const EdgeInsets.all(12),
      child: SizedBox(
        height: 32,
        child: TextField(
          controller: _searchController,
          onChanged: _onSearchChanged,
          decoration: InputDecoration(
            hintText: "搜索工具名称、描述或提供者...",
            hintStyle: TextStyle(
              fontSize: 12,
              color: Theme.of(context).hintColor,
            ),
            prefixIcon: Padding(
              padding: const EdgeInsets.all(8),
              child: FlowySvg(
                FlowySvgs.search_s,
                size: const Size.square(14),
                color: Theme.of(context).hintColor,
              ),
            ),
            suffixIcon: _searchKeyword.isNotEmpty
                ? IconButton(
                    icon: Icon(
                      Icons.clear,
                      size: 16,
                      color: Theme.of(context).hintColor,
                    ),
                    onPressed: () {
                      _searchController.clear();
                      _onSearchChanged('');
                    },
                  )
                : null,
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(6),
              borderSide: BorderSide(
                color: AFThemeExtension.of(context).borderColor,
                width: 1,
              ),
            ),
            enabledBorder: OutlineInputBorder(
              borderRadius: BorderRadius.circular(6),
              borderSide: BorderSide(
                color: AFThemeExtension.of(context).borderColor,
                width: 1,
              ),
            ),
            focusedBorder: OutlineInputBorder(
              borderRadius: BorderRadius.circular(6),
              borderSide: BorderSide(
                color: Theme.of(context).colorScheme.primary,
                width: 1,
              ),
            ),
            contentPadding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            isDense: true,
          ),
          style: const TextStyle(fontSize: 12),
        ),
      ),
    );
  }

  Widget _buildCategoryFilter() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      child: Row(
        children: [
          FlowyText.regular(
            "分类:",
            fontSize: 11,
            color: AFThemeExtension.of(context).textColor,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Wrap(
              spacing: 4,
              runSpacing: 4,
              children: [
                _buildCategoryChip(null, "全部"),
                ..._availableCategories.map(
                  (category) => _buildCategoryChip(category, category),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildCategoryChip(String? category, String label) {
    final isSelected = _selectedCategory == category;
    
    return GestureDetector(
      onTap: () => _onCategoryChanged(category),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          color: isSelected 
              ? Theme.of(context).colorScheme.primary.withOpacity(0.1)
              : AFThemeExtension.of(context).greyHover,
          borderRadius: BorderRadius.circular(12),
          border: isSelected
              ? Border.all(
                  color: Theme.of(context).colorScheme.primary,
                  width: 1,
                )
              : null,
        ),
        child: FlowyText.regular(
          label,
          fontSize: 10,
          color: isSelected 
              ? Theme.of(context).colorScheme.primary
              : AFThemeExtension.of(context).textColor,
        ),
      ),
    );
  }

  Widget _buildToolsList() {
    if (_filteredTools.isEmpty) {
      return Container(
        padding: const EdgeInsets.all(32),
        child: Column(
          children: [
            Icon(
              Icons.search_off,
              size: 32,
              color: Theme.of(context).hintColor,
            ),
            const SizedBox(height: 8),
            FlowyText.regular(
              _searchKeyword.isNotEmpty || _selectedCategory != null
                  ? "未找到匹配的工具"
                  : "暂无可用工具",
              fontSize: 12,
              color: Theme.of(context).hintColor,
            ),
          ],
        ),
      );
    }

    return Flexible(
      child: ListView.builder(
        shrinkWrap: true,
        itemCount: _filteredTools.length,
        itemBuilder: (context, index) {
          final tool = _filteredTools[index];
          return _buildToolItem(tool);
        },
      ),
    );
  }

  Widget _buildToolItem(log_entities.McpToolInfo tool) {
    final isSelected = _selectedToolIds.contains(tool.id);
    final statusInfo = _getToolStatusInfo(tool.status);
    
    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          borderRadius: BorderRadius.circular(6),
          onTap: () => _onToolToggled(tool.id, !isSelected),
          child: Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(6),
              border: isSelected
                  ? Border.all(
                      color: Theme.of(context).colorScheme.primary,
                      width: 1,
                    )
                  : null,
              color: isSelected
                  ? Theme.of(context).colorScheme.primary.withOpacity(0.05)
                  : null,
            ),
            child: Row(
              children: [
                // 选择框
                Container(
                  width: 16,
                  height: 16,
                  decoration: BoxDecoration(
                    borderRadius: BorderRadius.circular(3),
                    border: Border.all(
                      color: isSelected
                          ? Theme.of(context).colorScheme.primary
                          : AFThemeExtension.of(context).borderColor,
                      width: 1,
                    ),
                    color: isSelected
                        ? Theme.of(context).colorScheme.primary
                        : null,
                  ),
                  child: isSelected
                      ? Icon(
                          Icons.check,
                          size: 12,
                          color: Theme.of(context).colorScheme.onPrimary,
                        )
                      : null,
                ),
                const SizedBox(width: 12),
                
                // 工具信息
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Expanded(
                            child: FlowyText.medium(
                              tool.displayName ?? tool.name,
                              fontSize: widget.compactMode ? 12 : 13,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                            ),
                          ),
                          if (widget.showToolStatus) ...[
                            const SizedBox(width: 8),
                            _buildStatusIndicator(statusInfo),
                          ],
                        ],
                      ),
                      if (tool.description.isNotEmpty) ...[
                        const SizedBox(height: 2),
                        FlowyText.regular(
                          tool.description,
                          fontSize: widget.compactMode ? 10 : 11,
                          color: AFThemeExtension.of(context).textColor,
                          maxLines: widget.compactMode ? 1 : 2,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ],
                      if (!widget.compactMode) ...[
                        const SizedBox(height: 4),
                        Row(
                          children: [
                            if (tool.provider.isNotEmpty) ...[
                              _buildInfoChip(tool.provider, Icons.business),
                              const SizedBox(width: 6),
                            ],
                            if (tool.category.isNotEmpty) ...[
                              _buildInfoChip(tool.category, Icons.category),
                              const SizedBox(width: 6),
                            ],
                            if (widget.showToolStats && tool.usageCount > 0) ...[
                              _buildInfoChip(
                                "${tool.usageCount}次使用",
                                Icons.analytics,
                              ),
                            ],
                          ],
                        ),
                      ],
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildStatusIndicator(_ToolStatusInfo statusInfo) {
    return Container(
      width: 8,
      height: 8,
      decoration: BoxDecoration(
        color: statusInfo.color,
        shape: BoxShape.circle,
      ),
    );
  }

  Widget _buildInfoChip(String text, IconData icon) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(
        color: AFThemeExtension.of(context).greyHover,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            icon,
            size: 10,
            color: AFThemeExtension.of(context).textColor,
          ),
          const SizedBox(width: 2),
          FlowyText.regular(
            text,
            fontSize: 9,
            color: AFThemeExtension.of(context).textColor,
          ),
        ],
      ),
    );
  }

  Widget _buildFooter() {
    final selectedCount = _selectedToolIds.length;
    
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        border: Border(
          top: BorderSide(
            color: AFThemeExtension.of(context).borderColor,
            width: 1,
          ),
        ),
      ),
      child: Row(
        children: [
          Expanded(
            child: FlowyText.regular(
              selectedCount > 0 
                  ? "已选择 $selectedCount 个工具"
                  : "请选择要使用的工具",
              fontSize: 11,
              color: AFThemeExtension.of(context).textColor,
            ),
          ),
          FlowyButton(
            text: FlowyText.regular(
              "确定",
              fontSize: 12,
              color: Theme.of(context).colorScheme.primary,
            ),
            onTap: () => _popoverController.close(),
            hoverColor: Theme.of(context).colorScheme.primary.withOpacity(0.1),
          ),
        ],
      ),
    );
  }

  _ToolStatusInfo _getToolStatusInfo(log_entities.McpToolStatus status) {
    switch (status) {
      case log_entities.McpToolStatus.available:
      case log_entities.McpToolStatus.connected:
        return _ToolStatusInfo(
          color: AFThemeExtension.of(context).success ?? Colors.green,
          label: "可用",
        );
      case log_entities.McpToolStatus.connecting:
        return _ToolStatusInfo(
          color: Colors.orange,
          label: "连接中",
        );
      case log_entities.McpToolStatus.unavailable:
      case log_entities.McpToolStatus.disconnected:
        return _ToolStatusInfo(
          color: Theme.of(context).colorScheme.error,
          label: "不可用",
        );
      case log_entities.McpToolStatus.error:
        return _ToolStatusInfo(
          color: Theme.of(context).colorScheme.error,
          label: "错误",
        );
      case log_entities.McpToolStatus.unknown:
        return _ToolStatusInfo(
          color: AFThemeExtension.of(context).textColor,
          label: "未知",
        );
    }
  }
}

// 辅助数据类

class _ToolStatusInfo {
  const _ToolStatusInfo({
    required this.color,
    required this.label,
  });

  final Color color;
  final String label;
}
