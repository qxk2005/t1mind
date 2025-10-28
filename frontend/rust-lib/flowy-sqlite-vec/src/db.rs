use crate::entities::{
  EmbeddedContent, PendingIndexedCollab, SqliteEmbeddedDocument, SqliteEmbeddedFragment,
};
use crate::init_sqlite_vector_extension;
use crate::migration::init_sqlite_with_migrations;
use anyhow::{Context, Result};
use flowy_ai_pub::entities::{EmbeddedChunk, SearchResult};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{ToSql, params};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tracing::{info, trace, warn};
use uuid::Uuid;

pub struct VectorSqliteDB {
  pub pool: Pool<SqliteConnectionManager>,
}

impl VectorSqliteDB {
  pub fn new(root: PathBuf) -> Result<Self> {
    let db_path = root.join("vector.db");

    // 🔧 重要：必须在创建任何连接之前注册向量扩展
    init_sqlite_vector_extension();

    // Setup the connection manager with the database path
    let manager = SqliteConnectionManager::file(&db_path);

    // Initialize SQLite extensions and settings in each new connection
    let manager = manager.with_init(|_conn| {
      // 向量扩展已经通过 sqlite3_auto_extension 注册，会自动加载到每个新连接
      Ok(())
    });

    // Create the connection pool
    let pool = Pool::builder()
      .max_size(10) // Adjust based on your needs
      .build(manager)
      .context("Failed to create connection pool")?;

    // Ensure database is migrated
    let mut conn = pool
      .get()
      .context("Failed to get connection for migration")?;
    
    // 🔧 修复：如果迁移失败（例如 DatabaseTooFarAhead），尝试重建数据库
    if let Err(err) = init_sqlite_with_migrations(&mut conn) {
      warn!(
        "[Vector DB] Migration failed: {}. Will reset the database.",
        err
      );
      
      // 关闭连接
      drop(conn);
      drop(pool);
      
      // 删除旧数据库文件
      if db_path.exists() {
        std::fs::remove_file(&db_path)
          .context("Failed to remove old vector database file")?;
        warn!("[Vector DB] Removed old database file");
      }
      
      // 重新创建连接池和数据库
      // 向量扩展已经在函数开始时注册，不需要再次注册
      let manager = SqliteConnectionManager::file(&db_path);
      let manager = manager.with_init(|_conn| {
        // 向量扩展已经通过 sqlite3_auto_extension 注册
        Ok(())
      });
      
      let pool = Pool::builder()
        .max_size(10)
        .build(manager)
        .context("Failed to create connection pool after reset")?;
      
      let mut conn = pool
        .get()
        .context("Failed to get connection after reset")?;
      
      // 重新初始化
      init_sqlite_with_migrations(&mut conn)
        .context("Failed to migrate database after reset")?;
      
      warn!("[Vector DB] Database reset successfully");
      
      return Ok(Self { pool });
    }

    Ok(Self { pool })
  }

  pub async fn select_collabs_fragment_ids(
    &self,
    object_ids: &[String],
  ) -> Result<HashMap<Uuid, Vec<String>>> {
    if object_ids.is_empty() {
      return Ok(HashMap::new());
    }

    let placeholders = std::iter::repeat("?")
      .take(object_ids.len())
      .collect::<Vec<_>>()
      .join(", ");

    let sql = format!(
      "SELECT fragment_id, object_id FROM af_collab_embeddings WHERE object_id IN ({})",
      placeholders
    );

    let conn = self
      .pool
      .get()
      .context("Failed to get connection from pool")?;
    let mut stmt = conn
      .prepare(&sql)
      .context("Preparing select_collabs_fragment_ids")?;

    let params: Vec<&dyn ToSql> = object_ids.iter().map(|s| s as &dyn ToSql).collect();
    let mut rows = stmt
      .query(params.as_slice())
      .context("Executing select_collabs_fragment_ids")?;

    let mut map: HashMap<Uuid, Vec<String>> = HashMap::new();
    while let Some(row) = rows.next()? {
      let fragment_id: String = row.get(0)?;
      let oid_str: String = row.get(1)?;
      let oid = Uuid::parse_str(&oid_str)
        .map_err(|e| anyhow::anyhow!("Invalid UUID `{}` in DB: {}", oid_str, e))?;
      map.entry(oid).or_default().push(fragment_id);
    }

    Ok(map)
  }

  /// 统计指定文档的切片数量
  pub async fn count_fragments_for_document(
    &self,
    workspace_id: &str,
    object_id: &str,
  ) -> Result<usize> {
    let conn = self
      .pool
      .get()
      .context("Failed to get connection from pool")?;
    
    let count: i64 = conn
      .query_row(
        "SELECT COUNT(*) FROM af_collab_embeddings WHERE workspace_id = ? AND object_id = ?",
        params![workspace_id, object_id],
        |row| row.get(0),
      )?;
    
    Ok(count as usize)
  }

  /// 获取所有文档及其切片数量的统计信息
  pub async fn get_document_fragment_stats(
    &self,
    workspace_id: &str,
  ) -> Result<HashMap<String, usize>> {
    let conn = self
      .pool
      .get()
      .context("Failed to get connection from pool")?;
    
    let mut stmt = conn.prepare(
      "SELECT object_id, COUNT(*) as fragment_count 
       FROM af_collab_embeddings 
       WHERE workspace_id = ? 
       GROUP BY object_id"
    )?;
    
    let mut stats = HashMap::new();
    let mut rows = stmt.query(params![workspace_id])?;
    
    while let Some(row) = rows.next()? {
      let object_id: String = row.get(0)?;
      let count: i64 = row.get(1)?;
      stats.insert(object_id, count as usize);
    }
    
    Ok(stats)
  }

  pub async fn delete_collab(&self, workspace_id: &str, object_id: &str) -> Result<()> {
    let mut conn = self
      .pool
      .get()
      .context("Failed to get connection from pool")?;
    let tx = conn
      .transaction()
      .context("Starting delete_collab transaction")?;
    tx.execute(
      "DELETE FROM af_collab_embeddings
               WHERE workspace_id = ?1
                 AND object_id    = ?2",
      rusqlite::params![workspace_id, object_id],
    )
    .context("Deleting collab embeddings")?;

    tx.commit()
      .context("Committing delete_collab transaction")?;
    Ok(())
  }

  pub async fn select_all_embedded_content(
    &self,
    workspace_id: &str,
    rag_ids: &[String],
    limit: usize,
  ) -> Result<Vec<EmbeddedContent>> {
    let conn = self
      .pool
      .get()
      .context("Failed to get connection from pool")?;

    // Build SQL query based on whether rag_ids are provided
    let (sql, params) = if rag_ids.is_empty() {
      // No rag_ids provided, select all content for workspace
      let sql =
        "SELECT object_id, content FROM af_collab_embeddings WHERE workspace_id = ? LIMIT ?";
      let params: Vec<&dyn ToSql> = vec![&workspace_id, &limit];
      (sql.to_string(), params)
    } else {
      // Filter by provided rag_ids
      let placeholders = std::iter::repeat("?")
        .take(rag_ids.len())
        .collect::<Vec<_>>()
        .join(", ");

      let sql = format!(
        "SELECT object_id, content FROM af_collab_embeddings WHERE workspace_id = ? AND object_id IN ({}) LIMIT ?",
        placeholders
      );

      let mut params: Vec<&dyn ToSql> = vec![&workspace_id];
      params.extend(rag_ids.iter().map(|id| id as &dyn ToSql));
      params.push(&limit);
      (sql, params)
    };

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params.as_slice())?;

    let mut contents = Vec::new();
    while let Some(row) = rows.next()? {
      let object_id: String = row.get(0)?;
      let content: String = row.get(1)?;
      contents.push(EmbeddedContent { content, object_id });
    }

    Ok(contents)
  }

  pub async fn select_all_embedded_documents(
    &self,
    workspace_id: &str,
    rag_ids: &[String],
  ) -> Result<Vec<SqliteEmbeddedDocument>> {
    let conn = self
      .pool
      .get()
      .context("Failed to get connection from pool")?;

    // Build SQL query based on whether rag_ids are provided
    let (sql, params) = if rag_ids.is_empty() {
      // No rag_ids provided, select all documents for workspace
      let sql = "SELECT object_id, content, embedding 
                FROM af_collab_embeddings 
                WHERE workspace_id = ?";
      let params: Vec<&dyn ToSql> = vec![&workspace_id];
      (sql.to_string(), params)
    } else {
      // Filter by provided rag_ids
      let placeholders = std::iter::repeat("?")
        .take(rag_ids.len())
        .collect::<Vec<_>>()
        .join(", ");

      let sql = format!(
        "SELECT object_id, content, embedding 
         FROM af_collab_embeddings 
         WHERE workspace_id = ? AND object_id IN ({})",
        placeholders
      );

      let mut params: Vec<&dyn ToSql> = vec![&workspace_id];
      params.extend(rag_ids.iter().map(|id| id as &dyn ToSql));
      (sql, params)
    };

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params.as_slice())?;

    // Group results by object_id
    let mut documents_map: HashMap<String, Vec<SqliteEmbeddedFragment>> = HashMap::new();

    while let Some(row) = rows.next()? {
      let object_id: String = row.get(0)?;
      let content: String = row.get(1)?;

      // Convert embedding blob to Vec<f32>
      let embedding_blob: Vec<u8> = row.get(2)?;
      let embeddings = if !embedding_blob.is_empty() {
        // Convert bytes to Vec<f32> - each f32 is 4 bytes
        let mut vec = Vec::with_capacity(embedding_blob.len() / 4);
        for chunk in embedding_blob.chunks_exact(4) {
          if let Ok(array) = chunk.try_into() {
            vec.push(f32::from_le_bytes(array));
          }
        }
        vec
      } else {
        Vec::new()
      };

      // Add fragment to the corresponding object_id
      documents_map
        .entry(object_id)
        .or_default()
        .push(SqliteEmbeddedFragment {
          content,
          embeddings,
        });
    }

    // Convert the map to the required Vec<SqliteEmbeddedDocument>
    let documents = documents_map
      .into_iter()
      .map(|(object_id, fragments)| SqliteEmbeddedDocument {
        workspace_id: workspace_id.to_string(),
        object_id,
        fragments,
      })
      .collect();

    Ok(documents)
  }

  /// Inserts or replaces all of `fragments` for the given (workspace_id, object_id),
  /// deleting anything else in that scope first, and storing the new vector blobs.
  pub async fn upsert_collabs_embeddings(
    &self,
    workspace_id: &str,
    object_id: &str,
    fragments: Vec<EmbeddedChunk>,
  ) -> Result<()> {
    if fragments.is_empty() {
      return Ok(());
    }

    trace!(
      "[VectorStore] workspace:{} upserting {} fragments for {}",
      workspace_id,
      fragments.len(),
      object_id
    );

    let mut conn = self
      .pool
      .get()
      .context("Failed to get connection from pool")?;
    let tx = conn.transaction().context("Starting transaction")?;

    // 1) Collect new IDs
    let new_ids: Vec<&str> = fragments.iter().map(|f| f.fragment_id.as_str()).collect();

    // 2) Load existing IDs from the DB
    let mut stmt = tx.prepare(
      "SELECT fragment_id
           FROM af_collab_embeddings
          WHERE workspace_id = ?1
            AND object_id    = ?2",
    )?;
    let existing_ids: HashSet<String> = stmt
      .query_map(params![workspace_id, object_id], |row| row.get(0))?
      .collect::<Result<_, _>>()?;
    drop(stmt);

    // 3) Compute which to delete (existing − new)
    let to_delete: Vec<&str> = existing_ids
      .iter()
      .filter_map(|id| {
        if !new_ids.contains(&id.as_str()) {
          Some(id.as_str())
        } else {
          None
        }
      })
      .collect();

    // 4) Delete stale fragments (if any)
    if !to_delete.is_empty() {
      trace!(
        "[VectorStore] Deleting {} {} stale fragments",
        object_id,
        to_delete.len()
      );
      let placeholders = std::iter::repeat("?")
        .take(to_delete.len())
        .collect::<Vec<_>>()
        .join(", ");
      let sql = format!(
        "DELETE FROM af_collab_embeddings
               WHERE workspace_id = ?1
                 AND object_id    = ?2
                 AND fragment_id IN ({})",
        placeholders
      );
      let mut params: Vec<&dyn ToSql> = Vec::with_capacity(2 + to_delete.len());
      params.push(&workspace_id);
      params.push(&object_id);
      params.extend(to_delete.iter().map(|s| s as &dyn ToSql));
      tx.execute(&sql, params.as_slice())
        .context("Deleting stale fragments")?;
    }

    // 5) Insert only the brand-new fragments (new − existing)
    let to_insert: Vec<&EmbeddedChunk> = fragments
      .iter()
      .filter(|f| !existing_ids.contains(&f.fragment_id))
      .collect();

    if !to_insert.is_empty() {
      trace!(
        "[VectorStore] Inserting {} {} new fragments. ids: {:?}",
        object_id,
        to_insert.len(),
        to_insert
          .iter()
          .map(|f| f.fragment_id.as_str())
          .collect::<Vec<_>>()
      );
      let mut insert = tx.prepare(
        "INSERT INTO af_collab_embeddings
               (workspace_id, object_id, fragment_id,
                content_type, content, metadata,
                fragment_index, embedder_type, embedding)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
      )?;

      for frag in to_insert {
        // skip if content missing
        if frag.content.is_none() {
          continue;
        }
        // 将 Vec<f32> 转换为字节
        let embedding_bytes: Vec<u8> = if let Some(ref embeddings) = frag.embeddings {
          embeddings
            .iter()
            .flat_map(|&f| f.to_le_bytes())
            .collect()
        } else {
          Vec::new()
        };
        
        insert
          .execute(rusqlite::params![
            workspace_id,
            object_id,
            &frag.fragment_id,
            frag.content_type,
            frag.content.clone().unwrap_or_default(),
            frag.metadata,
            frag.fragment_index,
            frag.embedder_type,
            &embedding_bytes[..],
          ])
          .context("Inserting new fragment")?;
      }
    }

    tx.commit().context("Committing transaction")?;
    Ok(())
  }

  pub async fn search(
    &self,
    workspace_id: &str,
    object_ids: &[String],
    query: &[f32],
    top_k: i32,
  ) -> Result<Vec<SearchResult>> {
    self
      .search_with_score(workspace_id, object_ids, query, top_k, 0.4)
      .await
  }

  pub async fn search_with_score(
    &self,
    workspace_id: &str,
    object_ids: &[String],
    query: &[f32],
    top_k: i32,
    min_score: f32,
  ) -> Result<Vec<SearchResult>> {
    // clamp min_score to [0,1]
    let min_score = min_score.clamp(0.0, 1.0);
    if object_ids.is_empty() {
      self
        .search_without_object_ids(workspace_id, query, top_k, min_score)
        .await
    } else {
      self
        .search_with_object_ids(workspace_id, object_ids, query, top_k, min_score)
        .await
    }
  }

  async fn search_without_object_ids(
    &self,
    workspace_id: &str,
    query: &[f32],
    top_k: i32,
    min_score: f32,
  ) -> Result<Vec<SearchResult>> {
    trace!(
      "[VectorStore] Searching workspace:{} score:{}",
      workspace_id, min_score
    );
    // distance = 1 - score, so we only want distance <= max_distance
    let max_distance = 1.0 - min_score;
    // 将 &[f32] 转换为字节
    let query_blob: Vec<u8> = query
      .iter()
      .flat_map(|&f| f.to_le_bytes())
      .collect();

    let conn = self
      .pool
      .get()
      .context("Failed to get connection from pool")?;

    // k-NN MATCH without object_id filter
    let sql = "\
          SELECT
            object_id,
            content,
            metadata,
            distance,
            (1.0 - distance) AS score
          FROM af_collab_embeddings
          WHERE embedding MATCH ?
            AND k = ?
            AND workspace_id = ?
            AND distance <= ?
          ORDER BY distance ASC
      ";

    let mut stmt = conn.prepare(sql)?;
    let mut rows = stmt.query(params![
      query_blob,   // MATCH
      top_k,        // number of neighbors
      workspace_id, // workspace filter
      max_distance, // only distances ≤ this
    ])?;

    self.process_search_results(&mut rows)
  }

  async fn search_with_object_ids(
    &self,
    workspace_id: &str,
    object_ids: &[String],
    query: &[f32],
    top_k: i32,
    min_score: f32,
  ) -> Result<Vec<SearchResult>> {
    trace!(
      "[VectorStore] Searching workspace:{} with object_ids: {:?}, score:{}",
      workspace_id, object_ids, min_score
    );
    // distance = 1 - score, so we only want distance <= max_distance
    let max_distance = 1.0 - min_score;
    // 将 &[f32] 转换为字节
    let query_blob: Vec<u8> = query
      .iter()
      .flat_map(|&f| f.to_le_bytes())
      .collect();

    let conn = self
      .pool
      .get()
      .context("Failed to get connection from pool")?;

    // Create placeholders for the IN clause
    let placeholders = std::iter::repeat("?")
      .take(object_ids.len())
      .collect::<Vec<_>>()
      .join(", ");

    // k-NN MATCH with object_id filter
    let sql = format!(
      "SELECT
        object_id,
        content,
        metadata,
        distance,
        (1.0 - distance) AS score
      FROM af_collab_embeddings
      WHERE embedding MATCH ?
        AND k = ?
        AND workspace_id = ?
        AND object_id IN ({})
        AND distance <= ?
      ORDER BY distance ASC",
      placeholders
    );

    let mut stmt = conn.prepare(&sql)?;

    // Create the parameter vector with all params
    let mut query_params: Vec<&dyn ToSql> = Vec::with_capacity(4 + object_ids.len());
    query_params.push(&query_blob as &dyn ToSql);
    query_params.push(&top_k as &dyn ToSql);
    query_params.push(&workspace_id as &dyn ToSql);
    query_params.extend(object_ids.iter().map(|oid| oid as &dyn ToSql));
    query_params.push(&max_distance as &dyn ToSql);
    let mut rows = stmt.query(query_params.as_slice())?;
    self.process_search_results(&mut rows)
  }

  /// Process the query results and convert them to SearchResult objects
  fn process_search_results(&self, rows: &mut rusqlite::Rows) -> Result<Vec<SearchResult>> {
    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
      let oid_str: String = row.get(0)?;
      let oid = match Uuid::parse_str(&oid_str) {
        Ok(u) => u,
        Err(err) => {
          warn!("[VectorStore] Invalid UUID `{}` in DB: {}", oid_str, err);
          continue;
        },
      };
      let content: String = row.get(1)?;
      let metadata = row
        .get::<_, Option<String>>(2)?
        .and_then(|s| serde_json::from_str::<Value>(&s).ok());
      let score: f32 = row.get(4)?;
      trace!(
        "[VectorStore] Found {} embedding record, score: {}",
        oid, score
      );
      results.push(SearchResult {
        oid,
        content,
        metadata,
        score,
      });
    }

    Ok(results)
  }

  pub async fn delete_pending_indexed_collab(
    &self,
    workspace_id: &str,
    object_ids: Vec<String>,
  ) -> Result<()> {
    // Nothing to do if no object_ids provided
    if object_ids.is_empty() {
      return Ok(());
    }

    // Get connection from pool
    let mut conn = self
      .pool
      .get()
      .context("Failed to get connection from pool")?;
    let tx = conn
      .transaction()
      .context("Starting delete_pending_indexed_collab transaction")?;

    // Build the `IN (?, ?, …)` clause dynamically
    let placeholders = std::iter::repeat("?")
      .take(object_ids.len())
      .collect::<Vec<_>>()
      .join(", ");
    let sql = format!(
      "DELETE FROM af_pending_index_collab WHERE workspace_id = ?1 AND object_id IN ({})",
      placeholders
    );

    // Bind workspace_id as param 1, then each object_id
    let mut query_params: Vec<&dyn ToSql> = Vec::with_capacity(1 + object_ids.len());
    query_params.push(&workspace_id);
    for oid in &object_ids {
      query_params.push(oid as &dyn ToSql);
    }

    // Execute and commit
    tx.execute(&sql, query_params.as_slice())
      .context("Deleting pending indexed collabs")?;
    tx.commit()
      .context("Committing delete_pending_indexed_collab transaction")?;

    Ok(())
  }

  /// 重置向量数据库 - 清空所有嵌入数据
  /// 当嵌入模型维度发生变化时使用
  pub async fn reset_vector_database(&self) -> Result<()> {
    tracing::info!("[Vector DB] 🔄 开始重置向量数据库...");
    
    let mut conn = self
      .pool
      .get()
      .context("Failed to get connection from pool")?;
    let tx = conn
      .transaction()
      .context("Starting reset_vector_database transaction")?;

    // 清空所有嵌入数据
    tx.execute("DELETE FROM af_collab_embeddings", [])
      .context("Clearing af_collab_embeddings table")?;
    
    // 清空待索引数据
    tx.execute("DELETE FROM af_pending_index_collab", [])
      .context("Clearing af_pending_index_collab table")?;

    tx.commit()
      .context("Committing reset_vector_database transaction")?;

    tracing::info!("[Vector DB] ✅ 向量数据库重置完成");
    Ok(())
  }

  /// 获取当前嵌入向量的维度
  pub async fn get_embedding_dimension(&self) -> Result<usize> {
    let conn = self
      .pool
      .get()
      .context("Failed to get connection from pool")?;

    // 尝试从表结构获取维度
    let sql = "SELECT sql FROM sqlite_master WHERE type='table' AND name='af_collab_embeddings'";
    let mut stmt = conn.prepare(sql)?;
    let mut rows = stmt.query([])?;

    if let Some(row) = rows.next()? {
      let create_sql: String = row.get(0)?;
      // 解析 CREATE TABLE 语句中的 float[n] 格式
      if let Some(start) = create_sql.find("float[") {
        if let Some(end) = create_sql[start..].find(']') {
          let dimension_str = &create_sql[start + 6..start + end];
          if let Ok(dimension) = dimension_str.parse::<usize>() {
            info!("[Vector DB] 从表结构获取嵌入维度: {}", dimension);
            return Ok(dimension);
          }
        }
      }
    }

    // 如果无法从表结构获取，返回默认值
    warn!("[Vector DB] 无法从表结构获取嵌入维度，使用默认值 2560");
    Ok(2560)
  }

  /// 当嵌入模型维度发生根本性变化时使用
  pub async fn rebuild_vector_database(&self, embedding_dimension: usize) -> Result<()> {
    
    let mut conn = self
      .pool
      .get()
      .context("Failed to get connection from pool")?;
    let tx = conn
      .transaction()
      .context("Starting rebuild_vector_database transaction")?;

    // 删除旧的向量表
    tx.execute("DROP TABLE IF EXISTS af_collab_embeddings", [])
      .context("Dropping old af_collab_embeddings table")?;
    
    // 清空待索引数据
    tx.execute("DELETE FROM af_pending_index_collab", [])
      .context("Clearing af_pending_index_collab table")?;

    // 创建新的向量表，使用动态维度
    let create_table_sql = format!(
      "CREATE VIRTUAL TABLE af_collab_embeddings 
       USING vec0(
         workspace_id    TEXT    NOT NULL,
         object_id       TEXT    NOT NULL,
         fragment_id     TEXT    NOT NULL,
         content_type    INTEGER NOT NULL,
         content         TEXT    NOT NULL,
         metadata        TEXT,
         fragment_index  INTEGER NOT NULL DEFAULT 0,
         embedder_type   INTEGER NOT NULL DEFAULT 0,
         embedding       float[{}] 
       )",
      embedding_dimension
    );
    
    tx.execute(&create_table_sql, [])
      .context("Creating new af_collab_embeddings table with dynamic dimension")?;

    tx.commit()
      .context("Committing rebuild_vector_database transaction")?;

    tracing::info!("[Vector DB] ✅ 向量数据库重建完成，新维度: {}", embedding_dimension);
    Ok(())
  }

  pub async fn queue_pending_indexed_collab(&self, data: PendingIndexedCollab) -> Result<()> {
    self.batch_queue_pending_indexed_collabs(vec![data]).await
  }

  /// Enqueue multiple pending collabs in a single transaction.
  pub async fn batch_queue_pending_indexed_collabs(
    &self,
    data: Vec<PendingIndexedCollab>,
  ) -> Result<()> {
    if data.is_empty() {
      return Ok(());
    }

    // Get connection from pool
    let mut conn = self
      .pool
      .get()
      .context("Failed to get connection from pool")?;
    let tx = conn.transaction().context("Starting transaction")?;

    // prepare INSERT only once
    let mut stmt = tx.prepare(
      "INSERT INTO af_pending_index_collab
               (workspace_id, object_id, collab_type, content)
             VALUES (?1, ?2, ?3, ?4)",
    )?;

    // execute for each item
    for item in data {
      stmt
        .execute(rusqlite::params![
          item.workspace_id,
          item.object_id,
          item.collab_type,
          item.content,
        ])
        .context("Inserting pending collab")?;
    }

    // drop the Statement (releases borrow on tx) then commit
    drop(stmt);
    tx.commit().context("Committing transaction")?;
    Ok(())
  }
}
